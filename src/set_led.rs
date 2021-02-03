use std::error::Error;

use super::LED_PWM_FREQ;
use super::{Rocket,Status};

impl Rocket {
    pub fn set_led(&mut self) -> Result<(),Box<dyn Error>>{
        match self.current_status {
            Status::WaitForReload => {
                self.perf.led_red.set_duty_cycle(1.0)?;
                self.perf.led_green.set_duty_cycle(0.0)?;
                self.perf.led_blue.set_pwm_frequency(LED_PWM_FREQ,1.0)?;
            },
            Status::WaitForLaunch => {
                self.perf.led_red.set_duty_cycle(0.0)?;
                self.perf.led_green.set_duty_cycle(1.0)?;
                self.perf.led_blue.set_pwm_frequency(LED_PWM_FREQ,1.0)?;
            },
            Status::WaitForPeak => {
                self.perf.led_red.set_duty_cycle(0.0)?;
                self.perf.led_green.set_duty_cycle(1.0)?;
                self.perf.led_blue.set_pwm_frequency(LED_PWM_FREQ,0.0)?;
            },
            Status::WaitForLanding => {
                self.perf.led_red.set_duty_cycle(1.0)?;
                self.perf.led_green.set_duty_cycle(1.0)?;
                self.perf.led_blue.set_pwm_frequency(LED_PWM_FREQ,0.0)?;
            },
        }

        Ok(())
    }


}