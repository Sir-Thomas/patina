use crate::app_framework::prelude::*;
use crate::define_apps;

define_apps! {
    Alarm => alarm::AlarmApp,
    AppMenu => buoyant_app_menu::BuoyantAppMenuApp,
    Buoyant => buoyant::BuoyantApp,
    Clock => clock::ClockApp,
    HelloWorld => hello_world::HelloWorldApp,
    Settings => buoyant_settings::SettingsApp,
    Stopwatch => stopwatch::StopwatchApp,
}
