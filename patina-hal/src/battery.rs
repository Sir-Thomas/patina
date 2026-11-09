//! Battery Trait

#[allow(async_fn_in_trait)]
/// Battery
pub trait Battery {
    /// Returns true if battery is charging
    fn is_charging(&self) -> bool;
    /// Get battery percent 0-100
    async fn charge_level(&mut self) -> u8;
    /// Get battery voltage in millivolts
    async fn millivolts(&mut self) -> u32;
    /// Same as is_charging, but only returns on state change
    async fn wait_for_charge_state_change(&mut self) -> bool;
}
