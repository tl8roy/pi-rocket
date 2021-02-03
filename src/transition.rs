use std::error::Error;

use super::{Rocket,Status};

impl Rocket {
    pub fn transition(&mut self) -> Result<(),Box<dyn Error>>{
        match self.current_status {
            Status::WaitForReload => {
                //Button Press Or Timeout
                if /*self.perf.trigger.is_low() ||*/ self.flags.landing_reset.has_expired() {
                    self.current_status = Status::WaitForLaunch;
                    self.flags.status_change = true;
                }
            },
            Status::WaitForLaunch => {
                //G detection
                let accel_mag = self.current_accel.0.abs() + self.current_accel.1.abs() + self.current_accel.2.abs();
                if accel_mag > 3.0 {
                    self.current_status = Status::WaitForPeak;
                    self.flags.status_change = true;
                    self.log_start_index = Some(self.log.len() - 1);
                }
            },
            Status::WaitForPeak => {
                //Alt stable & MET is at least 10 seconds
                if let Some(ref met) = self.start_met  {
                    //Alt going down
                    if self.log.len() > 100  && met.elapsed().as_secs_f32() > 1.0 {

                        let mut last_alt = 0.0;
                        let mut last_time = 0.0;
                        
                        let mut sum = 0.0;
                        let mut count = 0.0;

                        //Work out the average gradient in the last 100 readings
                        for entry in self.log.iter().rev() {
                            if count > 1.0 {
                                sum += (last_alt - entry.alt) / (last_time - entry.met.as_secs_f32());
                                //println!("{} {}",(last_alt - entry.alt),(last_time - entry.met.as_secs_f32()));
                            }
                            

                            last_alt = entry.alt;
                            last_time = entry.met.as_secs_f32();

                            count += 1.0;
                            if count >= 100.0 {
                                break;
                            }
                        }
                        
                        //If the trend is down
                        if sum / 100.0 < -0.5 {

                            self.current_status = Status::WaitForLanding;
                            self.flags.status_change = true;
                        }
                    }
                }
            },
            Status::WaitForLanding => {
                //Alt stable & MET is at least 10 seconds
                if let Some(ref met) = self.start_met {
                    if self.log.len() > 20 && met.elapsed().as_secs_f32() > 5.0 {
                        let mut last_alt = 0.0;
                        let mut last_time = 0.0;
                        
                        let mut sum = 0.0;
                        let mut count = 0.0;

                        //Work out the average gradient in the last 20 readings
                        for entry in self.log.iter().rev() {
                            if count > 1.0 {
                                sum += (last_alt - entry.alt) / (last_time - entry.met.as_secs_f32());
                            }
                            

                            last_alt = entry.alt;
                            last_time = entry.met.as_secs_f32();

                            count += 1.0;
                            if count >= 20.0 {
                                break;
                            }
                        }

                        //println!("{} {}",sum,count);
                        if sum / 20.0 == 0.0 {
                            
                            self.current_status = Status::WaitForReload;
                            self.flags.status_change = true;
                        }
                    }
                }
            },
        }

        

        Ok(())
    }
}