use defmt::info;
use embassy_futures::select::{Either, select};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, watch::Receiver};
use embassy_time::{Duration, Timer};
use time::{PlainDateTime, SignedDuration, Time};

use crate::{
    app_framework::events::SystemEvent,
    signals::{CURRENT_TIME, EVENT_QUEUE, SET_ALARM},
};

#[embassy_executor::task]
pub async fn alarm_task() {
    let mut receiver = SET_ALARM.receiver().unwrap();
    let _ = receiver.changed().await;
    loop {
        if let (time, true) = receiver.get().await {
            info!("[Alarm] Set for: {:02}:{:02}", time.hour(), time.minute());
        } else {
            info!("[Alarm] Disabled");
        }
        match receiver.get().await {
            (_, false) => wait_for_change(&mut receiver).await,
            (time, true) => run_alarm(time, &mut receiver).await,
        }
    }
}

async fn wait_for_change(receiver: &mut Receiver<'static, ThreadModeRawMutex, (Time, bool), 1>) {
    let _ = receiver.changed().await;
}

async fn run_alarm(
    time: Time,
    receiver: &mut Receiver<'static, ThreadModeRawMutex, (Time, bool), 1>,
) {
    let current_time = CURRENT_TIME.try_get().unwrap();
    let mut alarm_time = PlainDateTime::new(current_time.date(), time);
    if alarm_time < current_time {
        alarm_time += SignedDuration::days(1);
    }
    let duration = alarm_time - CURRENT_TIME.try_get().unwrap();
    let result = select(
        receiver.changed(),
        Timer::after(Duration::from_millis(
            duration.whole_milliseconds().try_into().unwrap(),
        )),
    )
    .await;
    match result {
        Either::First(_) => {}
        Either::Second(_) => {
            EVENT_QUEUE.send(SystemEvent::Alarm).await;
            Timer::after(Duration::from_secs(60)).await;
        }
    }
}
