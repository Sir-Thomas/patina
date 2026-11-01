use crate::app_framework::prelude::*;
use crate::define_apps;

define_apps! {
    Accelerometer => accelerometer::AccelerometerApp,
    Alarm => alarm::AlarmApp,
    AlarmRing => alarm_ring::AlarmRing,
    AppMenu => app_menu::AppMenuApp,
    Buoyant => buoyant::BuoyantApp,
    Clock => clock::ClockApp,
    HeartRate => heart_rate::HeartRateApp,
    HelloWorld => hello_world::HelloWorldApp,
    Settings => settings::SettingsApp,
    Stopwatch => stopwatch::StopwatchApp,
}
