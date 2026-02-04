use crate::define_apps;
use crate::app_framework::prelude::*;

define_apps! {
    Clock => clock::ClockApp,
    Settings => settings::SettingsApp,
    Alarm => alarm::AlarmApp,
    Flashlight => flashlight::FlashlightApp,
    HelloWorld => hello_world::HelloWorldApp,
    Stopwatch => stopwatch::StopwatchApp,
}