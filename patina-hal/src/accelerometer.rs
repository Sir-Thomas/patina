//! Accelerometer Trait

#[allow(async_fn_in_trait)]
/// Accelerometer
/// I should eventually fix this so it is impossible to call functions from invalid states, but the
/// accelerometer is not a high priority right now.
pub trait Accelerometer {
    /// Initialize Accelerometer
    async fn init(&mut self);
    /// Read value
    /// You must call start first, otherwise this will return (0, 0, 0)
    async fn read(&mut self) -> (i16, i16, i16);
    /// Start Accelerometer
    /// You must call init first
    async fn start(&mut self);
    /// Stop Accelerometer
    /// Your must call init first
    async fn stop(&mut self);
}
