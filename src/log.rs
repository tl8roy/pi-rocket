use std::error::Error;

use super::{Rocket,Status,LogEntry};

impl Rocket {
    pub fn log(&mut self) -> Result<(),Box<dyn Error>>{

        let log_entry = LogEntry {
            status: self.current_status,
            met: self.current_met.elapsed(),
            alt: self.current_alt,
            accel: self.current_accel,
        };

        //Add Log Entry
        self.log.push(log_entry);

        if self.current_status == Status::WaitForLaunch {
            //Cycle log
            if self.log.len() >= 1000 {
                self.log.remove(0);
            }
            
        }


        Ok(())
    }

}