use embedded_graphics_simulator::{
    SimulatorDisplay, Window, OutputSettingsBuilder
};

fn main() {
    let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(240, 240));
    let output_settings = OutputSettingsBuilder::new().build();
    let mut window = Window::new("App Preview", &output_settings);
    
    
    
    window.update(&display);
}