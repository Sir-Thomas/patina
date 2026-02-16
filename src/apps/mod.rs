use crate::define_apps;
use crate::app_framework::prelude::*;

define_apps! {
    Alarm => alarm::AlarmApp,
    AppMenu => app_menu::AppMenuApp,
    Clock => clock::ClockApp,
    HelloWorld => hello_world::HelloWorldApp,
    Settings => settings::SettingsApp,
    Stopwatch => stopwatch::StopwatchApp,
}