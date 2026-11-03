use crate::{
    app_framework::{AppContext, EventResponse, context::Command, events::SystemEvent},
    apps::{AppId, AppInstance},
    tasks::{self, ble::ble_runner},
};
use defmt::{debug, info};
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use heapless::Vec;
use nrf_sdc::mpsl::Flash;
use patina_hal::{ScreenOrientation, display::Display};
use pinetime_bsp::{Accelerometer, PineTime, backlight::BacklightController, vibrator::Vibrator};

use crate::signals::{CHANGE_DISPLAY_STATE, EVENT_QUEUE, FLIP_SCREEN, REFRESH_TIMEOUT};

#[embassy_executor::task]
pub async fn app_manager(spawner: Spawner, board: PineTime) {
    info!("[App Manager] Creating event receiver");
    let receiver = EVENT_QUEUE.receiver();
    info!("[App Manager] Initializing App Manager");
    let mut app_manager = AppManager::new(
        board.accelerometer,
        board.backlight,
        board.display,
        board.mpsl_flash,
        board.vibrator,
    );
    info!("[App Manager] Spawning Alarm task");
    spawner.spawn(tasks::alarm::alarm_task().unwrap());
    info!("[App Manager] Spawning Battery task");
    spawner.spawn(tasks::battery::battery_task(board.battery).unwrap());
    info!("[App Manager] Spawning button task");
    spawner.spawn(tasks::button::button_task(board.button).unwrap());
    info!("[App Manager] Spawning touch task");
    spawner.spawn(tasks::touch::touch_task(board.touchscreen).unwrap());
    info!("[App Manager] Spawning display timeout task");
    spawner.spawn(tasks::display_timeout::display_timeout_task().unwrap());
    info!("[App Manager] Spawning BLE tasks");
    spawner.spawn(ble_runner(board.bluetooth, board.spi_flash.flash, spawner).unwrap());
    info!("[App Manager] All tasks spawned, waiting for first event");
    // Wait for first systick to ensure clock is available
    receiver.receive().await;
    receiver.receive().await;
    app_manager.init().await;
    info!("[App Manager] Starting event loop");

    loop {
        let event = receiver.receive().await;
        debug!("[App Manager] Received Event: {:?}", event);
        app_manager.handle_event(event).await;
    }
}

struct AppManager<D> {
    app_id: AppId,
    context: AppContext,
    current_app: AppInstance,
    display: D,
}

impl<D: Display> AppManager<D> {
    fn new(
        accelerometer: Accelerometer,
        backlight: BacklightController,
        display: D,
        flash: &'static Mutex<NoopRawMutex, Flash<'static>>,
        vibrator: Vibrator,
    ) -> Self {
        let app_id = AppId::Clock;
        Self {
            app_id,
            context: AppContext::new(accelerometer, backlight, flash, vibrator),
            current_app: AppInstance::new(app_id),
            display,
        }
    }

    async fn init(&mut self) {
        self.clear_display().await;
        self.turn_on_display().await;
        self.current_app.on_start(&mut self.context).await;
        self.draw_screen().await;
    }

    async fn handle_event(&mut self, event: SystemEvent) {
        // Events that act the same with the display on or off
        match event {
            SystemEvent::Alarm => {
                info!("[Alarm] Ringing");
                self.current_app.on_stop(&mut self.context).await;
                self.current_app = AppInstance::new(AppId::AlarmRing);
                self.app_id = AppId::AlarmRing;
                self.draw_screen().await;
                if self.context.display_is_off() {
                    self.turn_on_display().await;
                    REFRESH_TIMEOUT.signal(());
                }
                self.current_app.on_start(&mut self.context).await;
            }
            SystemEvent::BluetoothConnected => {
                info!("[Bluetooth] Connected");
                self.context.set_bluetooth_connected(true);
            }
            SystemEvent::BluetoothDisconnected => {
                info!("[Bluetooth] Disconnected");
                self.context.set_bluetooth_connected(false);
            }
            _ => {}
        }

        // Event handling with display off
        if self.context.display_is_off() {
            match event {
                SystemEvent::ButtonPress | SystemEvent::Touch(_) => {
                    self.turn_on_display().await;
                    self.current_app.on_start(&mut self.context).await;
                    self.draw_screen().await;
                    REFRESH_TIMEOUT.signal(());
                }
                _ => {}
            }
            return;
        }

        // Event handling with display off
        match event {
            SystemEvent::ScreenTimeout => {
                self.switch_to(AppId::Clock).await;
                self.current_app.on_stop(&mut self.context).await;
                self.turn_off_display().await;
                return;
            }
            SystemEvent::ButtonPress => {
                self.close_current_app().await;
                return;
            }
            SystemEvent::Tick => {}
            _ => REFRESH_TIMEOUT.signal(()),
        }

        let response = self.current_app.on_event(event, &mut self.context).await;

        match response {
            EventResponse::CloseApp => {
                self.close_current_app().await;
            }
            EventResponse::Rerender => {
                self.draw_screen().await;
            }
            EventResponse::SwitchApp(new_id) => {
                self.switch_to(new_id).await;
            }
            EventResponse::Ignore => {}
        }

        for command in self.context.commands.clone() {
            self.run_command(command).await;
        }
        self.context.commands = Vec::new();
    }

    async fn run_command(&mut self, command: Command) {
        match command {
            Command::RotateScreen => self.rotate_screen().await,
        }
    }

    async fn switch_to(&mut self, new_id: AppId) {
        self.current_app.on_stop(&mut self.context).await;
        self.current_app = AppInstance::new(new_id);
        self.app_id = new_id;
        self.current_app.on_start(&mut self.context).await;
        self.draw_screen().await;
    }

    async fn close_current_app(&mut self) {
        if self.app_id != AppId::Clock {
            self.clear_display().await;
            self.switch_to(AppId::Clock).await;
        } else {
            self.current_app.on_stop(&mut self.context).await;
            self.turn_off_display().await;
        }
    }

    pub async fn turn_on_display(&mut self) {
        info!("[Context] Turning on display");
        CHANGE_DISPLAY_STATE.signal(true);
        self.context.screen_on = true;
        self.context.backlight.enable();
        self.display.wake().await;
    }

    pub async fn turn_off_display(&mut self) {
        info!("[Context] Turning off display");
        CHANGE_DISPLAY_STATE.signal(false);
        self.context.screen_on = false;
        self.context.backlight.disable();
        self.clear_display().await;
        self.display.sleep().await;
    }

    pub async fn clear_display(&mut self) {
        info!("[Context] Clearing display");
        self.display.fill(Rgb565::BLACK).await;
    }

    /*
    // TODO: Delete these if I still don't need them
    pub async fn fill_display(&mut self, color: Rgb565) {
        info!("[Context] Filling display with color");
        self.display.fill(color).await;
    }

    pub async fn draw<T: Drawable<Color = Rgb565> + Dimensions>(
        &mut self,
        drawable: &T,
        background: Rgb565,
    ) {
        self.display.draw(drawable, background).await;
    }
    */

    pub async fn draw_screen(&mut self) {
        self.current_app
            .render(&mut self.context, &mut self.display)
            .await;
    }

    pub async fn rotate_screen(&mut self) {
        info!("[Context] Flipping screen");
        FLIP_SCREEN.signal(());
        match self.display.orientation() {
            ScreenOrientation::Normal => {
                self.display
                    .set_orientation(ScreenOrientation::Flipped)
                    .await
            }
            ScreenOrientation::Flipped => {
                self.display
                    .set_orientation(ScreenOrientation::Normal)
                    .await
            }
        }
    }
}
