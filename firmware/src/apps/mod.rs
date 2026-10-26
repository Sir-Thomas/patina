use crate::app_framework::prelude::*;
use crate::define_apps;

define_apps! {
    Alarm => alarm::AlarmApp,
    AppMenu => app_menu::AppMenuApp,
    Buoyant => buoyant::BuoyantApp,
    Clock => clock::ClockApp,
    HelloWorld => hello_world::HelloWorldApp,
    Settings => settings::SettingsApp,
    Stopwatch => stopwatch::StopwatchApp,
}
