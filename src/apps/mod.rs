use crate::define_apps;
use crate::app_framework::prelude::*;

define_apps! {
    Clock => clock::ClockApp,
    Settings => settings::SettingsApp,
    Alarm => alarm::AlarmApp,
    HelloWorld => hello_world::HelloWorldApp,
    Stopwatch => stopwatch::StopwatchApp,
}