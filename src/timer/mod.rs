// ---------------------------------------------------------
// Author: Tristan Lostroh
// Module: Logic Controller
// What it has: As small module that provides a basic polled timer
// What it does: Takes a duration and once that duration has elapsed from the start, has_expired() will return true.
//              To work, this needs to be polled. The polling rate will determine the resolution.
// ---------------------------------------------------------

use std::time::{Duration, Instant};

#[derive(Debug,Clone)]
pub struct Timer {
    timer_start: Instant,
    pub target_duration: Duration
}

impl Timer {
    pub fn new(target_duration: Duration) -> Self {
        Timer {
            timer_start: Instant::now(),
            target_duration: target_duration
        }
    }

    pub fn has_expired(&self) -> bool {
        self.timer_start.elapsed() >= self.target_duration
    }

    /*pub fn elapsed(&self) -> Duration {
        self.timer_start.elapsed()
    }*/
}


#[test]
fn timer_test() {
    use std::thread::sleep;

    let duration_test = Duration::from_millis(10);
    let timer = Timer::new(duration_test);
    assert_eq!(false,timer.has_expired());
    sleep(duration_test);
    assert_eq!(true,timer.has_expired());

    let duration_test = Duration::from_millis(100);
    let timer = Timer::new(duration_test);
    assert_eq!(false,timer.has_expired());
    sleep(duration_test);
    assert_eq!(true,timer.has_expired());

    let duration_test = Duration::from_millis(1000);
    let timer = Timer::new(duration_test);
    assert_eq!(false,timer.has_expired());
    sleep(duration_test);
    assert_eq!(true,timer.has_expired());
}