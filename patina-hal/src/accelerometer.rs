//! Accelerometer Trait

/// Accelerometer
pub trait Accelerometer {
    async fn init(&mut self);

    async fn read(&mut self) -> (i16, i16, i16);

    async fn start(&mut self);

    async fn stop(&mut self);
}
