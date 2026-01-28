use crate::define_apps;
use crate::app_framework::prelude::*;

define_apps! {
    Clock => clock::ClockApp,
    Alarm => alarm::AlarmApp,
    Sample => sample::SampleApp,
    Stopwatch => stopwatch::StopwatchApp,
    Settings => settings::SettingsApp,
}