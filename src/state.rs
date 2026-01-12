use defmt::info;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use embedded_graphics::prelude::Point;

use crate::{button::{BUTTON_SIGNAL, ButtonAction}, display::Backlight};


pub static BACKLIGHT_SIGNAL: Signal<ThreadModeRawMutex, Backlight> = Signal::new();

pub enum Event {
    ButtonPress,
    ButtonLongPress,
    Touch(Point),
    SystemTick,
}

pub enum Screen {
    Clock,
    Applications,
    Settings,
}

pub struct PatinaState {
    pub current_screen: Screen,
    pub backlight_state: Backlight,
}

impl PatinaState {
    pub fn handle_event(&mut self, event: Event) {
        if let Event::ButtonPress = event {
            self.toggle_backlight();
            return;
        }
    }

    fn toggle_backlight(&mut self) {
        self.backlight_state = !self.backlight_state;
        info!("Backlight toggled");
        BACKLIGHT_SIGNAL.signal(self.backlight_state);
    }
}

#[embassy_executor::task]
pub async fn state_machine_task() {
    let mut patina_state = PatinaState{ current_screen: Screen::Clock, backlight_state: Backlight::On };
    loop {
        let button_signal = BUTTON_SIGNAL.wait().await;
        info!("STATE: Button event received");
        match button_signal {
            ButtonAction::Press => patina_state.handle_event(Event::ButtonPress),
            _ => {}
        }
    }
}