use crate::app_framework::{AppContext, EventResponse};
use crate::app_framework::events::SystemEvent;
use crate::apps::{AppId, AppInstance};
use crate::tasks;
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
    // Wait for first systick to ensure clock is available
    receiver.receive().await;
    app_manager.init().await;
    info!("[App Manager] Spawning button task");
    spawner.must_spawn(tasks::button::button_task(board.button));
    info!("[App Manager] Spawning touch task");
    spawner.must_spawn(tasks::touch::touch_task(board.touchscreen));
    info!("[App Manager] Spawning display timeout task");
    spawner.must_spawn(tasks::display_timeout::display_timeout_task());
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
        self.context.turn_on_display();
        self.current_app.on_start(&mut self.context).await;
        self.current_app.render(&mut self.context).await;
    }

    async fn handle_event(&mut self, event: SystemEvent) {
        if self.context.display_is_off() {
            if event == SystemEvent::ButtonPress {
                self.current_app.on_start(&mut self.context).await;
                self.current_app.render(&mut self.context).await;
                self.context.turn_on_display();
                REFRESH_TIMEOUT.signal(());
            }
            return;
        }

        // TODO: Consider changing ux so button always goes back to clock
        match event {
            SystemEvent::ScreenTimeout => {
                self.current_app.on_stop(&mut self.context).await;
                self.context.turn_off_display();
                return;
            },
            _ => REFRESH_TIMEOUT.signal(()),
        }
            
        let response = self.current_app.on_event(
            event, 
            &mut self.context
        ).await;
            
        match response {
            EventResponse::CloseApp => {
                if self.app_id != AppId::Clock {
                    self.switch_to(AppId::Clock).await;
                } else {
                    self.current_app.on_stop(&mut self.context).await;
                    self.context.turn_off_display();
                }
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
}