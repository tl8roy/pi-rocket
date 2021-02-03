use std::error::Error;
use std::time::Duration;

use accelerometer::Accelerometer;

use super::{Rocket,Status,Timer, ALT_FREQ, ACCEL_FREQ};

impl Rocket {
    pub fn doing(&mut self) -> Result<(),Box<dyn Error>>{
        match self.current_status {
            Status::WaitForReload => {
                //Nothing
            },
            Status::WaitForLaunch => {
                //Check for new G reading
                if self.flags.new_accel.has_expired() {
                    let all = self.perf.mma8452q.accel_norm().expect("Acc Sensor Error 7");
                    //(x,y,z)
                    self.current_accel = (all.x,all.y,all.z);

                    //Maintain Log
                    self.log()?;

                    self.flags.new_accel = Timer::new(Duration::from_micros(ACCEL_FREQ));
                }
            },
            Status::WaitForPeak | Status::WaitForLanding => {
                //Check for new G reading
                if self.flags.new_accel.has_expired() {
                    let all = self.perf.mma8452q.accel_norm().expect("Acc Sensor Error 8");
                    //(x,y,z)
                    self.current_accel = (all.x,all.y,all.z);

                    //Maintain Log
                    self.log()?;

                    self.flags.new_accel = Timer::new(Duration::from_micros(ACCEL_FREQ));
                }

                //Check for new alt reading
                if self.flags.new_alt.has_expired() {
                    self.perf.mpl3115.start_reading().expect("Alt Sensor Error 4");
                    
                    self.flags.waiting_for_alt = true;

                    self.flags.new_alt = Timer::new(Duration::from_millis(ALT_FREQ)); 
                }

                if self.flags.waiting_for_alt {

                    if self.perf.mpl3115.check_temp_reading().expect("Alt Sensor Error 5") {
                        self.current_alt = self.perf.mpl3115.get_pa_reading().expect("Alt Sensor Error 6");

                        self.flags.waiting_for_alt = false;

                        //Maintain Log
                        self.log()?;
                    }
                    
                }

            },

        }

        Ok(())
    }
}