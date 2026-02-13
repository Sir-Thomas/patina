use crate::app_framework::{AppContext, EventResponse};
use crate::app_framework::events::SystemEvent;
use crate::apps::{AppId, AppInstance};
use crate::tasks;
use crate::tasks::ble::ble_runner;
use defmt::{debug, info};
use embassy_executor::Spawner;
use pinetime_bsp::PineTime;
use pinetime_bsp::backlight::BacklightController;
use pinetime_bsp::display::DisplayController;
use pinetime_bsp::vibrator::Vibrator;

use crate::signals::{EVENT_QUEUE, REFRESH_TIMEOUT};

#[embassy_executor::task]
pub async fn app_manager(spawner: Spawner, board: PineTime) {
    info!("[App Manager] Creating event receiver");
    let receiver = EVENT_QUEUE.receiver();
    info!("[App Manager] Initializing App Manager");
    let mut app_manager = AppManager::new(board.backlight, board.display, board.vibrator);
    info!("[App Manager] Spawning Battery task");
    spawner.must_spawn(tasks::battery::battery_task(board.battery));
    info!("[App Manager] Spawning button task");
    spawner.must_spawn(tasks::button::button_task(board.button));
    info!("[App Manager] Spawning touch task");
    spawner.must_spawn(tasks::touch::touch_task(board.touchscreen));
    info!("[App Manager] Spawning display timeout task");
    spawner.must_spawn(tasks::display_timeout::display_timeout_task());
    info!("[App Manager] Spawning BLE tasks");
    spawner.must_spawn(ble_runner(board.bluetooth, board.spi_flash.flash, spawner));
    info!("[App Manager] All tasks spawned, waiting for first event");
    // Wait for first systick to ensure clock is available
    receiver.receive().await;
    app_manager.init().await;
    info!("[App Manager] Starting event loop");

    loop {
        let event = receiver.receive().await;
        debug!("[App Manager] Received Event: {:?}", event);
        app_manager.handle_event(event).await;
    }
}

struct AppManager {
    app_id: AppId,
    context: AppContext,
    current_app: AppInstance,
}

impl AppManager {
    fn new(
        backlight: BacklightController,
        display: DisplayController,
        vibrator: Vibrator,
    ) -> Self {
        let app_id = AppId::Clock;
        Self {
            app_id,
            context: AppContext::new(
                backlight,
                display,
                vibrator,
            ),
            current_app: AppInstance::new(app_id),
        }
    }

    async fn init(&mut self) {
        self.context.clear_display().await;
        self.context.turn_on_display().await;
        self.current_app.on_start(&mut self.context).await;
        self.current_app.render(&mut self.context).await;
    }

    async fn handle_event(&mut self, event: SystemEvent) {
        if self.context.display_is_off() {
            match event {
                SystemEvent::ButtonPress => {
                    self.context.turn_on_display().await;
                    self.current_app.on_start(&mut self.context).await;
                    self.current_app.render(&mut self.context).await;
                    REFRESH_TIMEOUT.signal(());
                }
                SystemEvent::BluetoothConnected => {
                    self.context.set_bluetooth_connected(true);
                }
                SystemEvent::BluetoothDisconnected => {
                    self.context.set_bluetooth_connected(false);
                }
                _ => {}
            }
            return;
        }

        match event {
            SystemEvent::ScreenTimeout => {
                self.current_app.on_stop(&mut self.context).await;
                self.context.turn_off_display().await;
                return;
            },
            SystemEvent::ButtonPress => {
                self.close_current_app().await;
                return;
            }
            SystemEvent::BluetoothConnected => {
                self.context.set_bluetooth_connected(true);
            }
            SystemEvent::BluetoothDisconnected => {
                self.context.set_bluetooth_connected(false);
            }
            SystemEvent::Tick => {},
            _ => REFRESH_TIMEOUT.signal(()),
        }
            
        let response = self.current_app.on_event(
            event, 
            &mut self.context
        ).await;
            
        match response {
            EventResponse::CloseApp => {
                self.close_current_app().await;
            }
            EventResponse::Rerender => {
                self.current_app.render(&mut self.context).await;
            }
            EventResponse::SwitchApp(new_id) => {
                self.switch_to(new_id).await;
            }
            EventResponse::Ignore => {}
        }
    }
    
    async fn switch_to(&mut self, new_id: AppId) {
        self.current_app.on_stop(&mut self.context).await;
        self.current_app = AppInstance::new(new_id);
        self.app_id = new_id;
        self.current_app.on_start(&mut self.context).await;
        self.current_app.render(&mut self.context).await;
    }

    async fn close_current_app(&mut self) {
        if self.app_id != AppId::Clock {
            self.context.clear_display().await;
            self.switch_to(AppId::Clock).await;
        } else {
            self.current_app.on_stop(&mut self.context).await;
            self.context.turn_off_display().await;
        }
    }
}