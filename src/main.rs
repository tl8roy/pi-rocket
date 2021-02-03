
use std::time::{Instant,Duration};
use std::io::prelude::*;
use std::fs::{OpenOptions};
use std::thread;
use std::fmt;
use std::error::Error;

use rppal::gpio::{Gpio, InputPin, OutputPin};
use rppal::pwm::{Channel, Polarity, Pwm};
use rppal::i2c::I2c;
//use rppal::system::DeviceInfo;
use accelerometer;

use strum_macros::{Display};
use chrono::prelude::*;
use mpl3115::*;
use mma8452q::*;

mod set_led;
mod transition;
mod doing;
mod log;
mod timer;
use timer::Timer;

// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
//const GPIO_LED_RED: u8 = 21;//27
//const GPIO_LED_GREEN: u8 = 17;
const GPIO_LED_BLUE: u8 = 4;
pub const LED_PWM_FREQ: f64 = 100.0;

const GPIO_SOLENOID: u8 = 17;
const GPIO_TRIGGER: u8 = 27;

pub const ACCEL_FREQ: u64 = 2500;
pub const ALT_FREQ: u64 = 50;



struct Rocket {
    start_met: Option<Instant>,
    current_status: Status,
    current_met: Instant,
    current_alt: f32,
    current_accel: (f32,f32,f32),
    log: Vec<LogEntry>,
    log_start_index: Option<usize>,
    flags: RocketFlags,
    perf: RocketPerf,
}

#[derive(Debug,Clone)]
struct RocketFlags {
    //New mode
    status_change: bool,

    //New alt
    new_alt: Timer,
    waiting_for_alt: bool,

    //New accel
    new_accel: Timer,

    landing_reset: Timer,
}

struct RocketPerf {
    //i2c
    mpl3115: MPL3115A2<I2c>,
    mma8452q: MMA8452q<I2c>,
        //pres
        //accel
        //rtc

    led_red: Pwm,
    led_green: Pwm,
    led_blue: OutputPin,
    solenoid: OutputPin,
    trigger: InputPin,

    //alt
}

#[derive(Debug,Clone,Copy)]
struct LogEntry {
    status: Status,
    met: Duration,
    alt: f32,
    accel: (f32,f32,f32),
}

impl fmt::Display for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:2.3}, {}, {:2.1}, {:1.2}, {:1.2}, {:1.2}", self.met.as_secs_f64(), self.status, self.alt, self.accel.0, self.accel.1, self.accel.2)
    }
}

#[derive(Debug,Clone,Copy,Display,PartialEq)]
enum Status {
    WaitForReload,
    WaitForLaunch,
    WaitForPeak,
    WaitForLanding,

}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting");

    //File names using session date
    let local: DateTime<Local> = Local::now();
    let local = local.format("%Y-%m-%d_%H:%M").to_string();
    let log_file = local.clone()+".csv";
    let temp_file = local+"_temp.csv";


    //Set up the controller
    //i2c
    let i2c = I2c::new()?; 

    let mut mpl3115 = MPL3115A2::new(i2c,PressureAlt::Altitude).expect("Alt Sensor Error 1");

    mpl3115.activate().expect("Alt Sensor Error 2");
 
    mpl3115.set_oversample_rate(3).expect("Alt Sensor Error 3");

    let i2c = I2c::new()?; 

    let mut mma8452q = MMA8452q::new(i2c,SlaveAddr::Alternative(true)).expect("Acc Sensor Error 1");

    mma8452q.standby().expect("Acc Sensor Error 2");
 
    mma8452q.set_mode(mma8452q::Mode::HighResolution).expect("Acc Sensor Error 3");
    mma8452q.set_odr(Odr::Hz400).expect("Acc Sensor Error 4");
    mma8452q.set_fs(FullScale::G8).expect("Acc Sensor Error 5");
    mma8452q.active().expect("Acc Sensor Error 6");

    println!("I2C Booted");

    //GPIO
    let led_red = Pwm::with_frequency(Channel::Pwm0, LED_PWM_FREQ, 0.0, Polarity::Normal, true)?;
    let led_green = Pwm::with_frequency(Channel::Pwm1, LED_PWM_FREQ, 0.0, Polarity::Normal, true)?;
    let mut led_blue = Gpio::new()?.get(GPIO_LED_BLUE)?.into_output();

    led_red.set_duty_cycle(0.0)?;
    led_green.set_duty_cycle(0.0)?;
    led_blue.set_pwm_frequency(LED_PWM_FREQ,0.0)?;

    let mut solenoid = Gpio::new()?.get(GPIO_SOLENOID)?.into_output();
    solenoid.set_low();

    let trigger = Gpio::new()?.get(GPIO_TRIGGER)?.into_input_pullup();

    //Camera
    use rascam::SimpleCamera;
    let info = rascam::info().expect("Camera 1");
    //println!("{}",info);
    let mut camera = SimpleCamera::new(info.cameras[0].clone()).expect("Camera 2");

    camera.activate().expect("Camera 3");

    println!("GPIO Booted");

    let mut rocket = Rocket {
        start_met: None,
        current_status: Status::WaitForReload,
        current_met: Instant::now(),
        current_alt: 0.0,
        current_accel: (0.0,0.0,0.0),//fn acceleration(&mut self) -> Result<I16x3, Error<E>> {
        log: Vec::with_capacity(1000),
        log_start_index: None,
        flags: RocketFlags {
            status_change: true,
            new_alt: Timer::new(Duration::from_secs(100)),
            waiting_for_alt: false,
            new_accel: Timer::new(Duration::from_secs(100)),
            landing_reset: Timer::new(Duration::from_secs(100)),
        },
        perf: RocketPerf {
            //i2c
            mpl3115: mpl3115,
            mma8452q: mma8452q,
                //accel
                //rtc
    
            led_red: led_red,
            led_green: led_green,
            led_blue: led_blue,
            solenoid: solenoid,
            trigger: trigger,
    
            //alt
        },
    };

    println!("Lets Go!");

    loop {
 
        if rocket.flags.status_change {
            println!("{}",rocket.current_status);
            match rocket.current_status {
                Status::WaitForReload => {
                    //Dump to file
                    if let Some(log_start_index) = rocket.log_start_index {

                        let mut file = OpenOptions::new().append(true).create(true).open(log_file.as_str())?;

                        for (index,entry) in rocket.log.iter().enumerate() {
                            if index >=  log_start_index {
                                writeln!(file,"{}",entry)?;
                            }
                        }
                    }

                    //Clear Log
                    rocket.log = Vec::with_capacity(1000);
                    rocket.log_start_index = None;

                    //Reset the MET
                    rocket.current_met = Instant::now();

                    //Reset Solenoid
                    rocket.perf.solenoid.set_low();

                    //Start reload timer
                    rocket.flags.landing_reset = Timer::new(Duration::from_secs(40));

                    //Reset the MET
                    rocket.start_met = None;
                },
                Status::WaitForLaunch => {
                    //Start interrupts for accel/timer
                    rocket.flags.new_alt = Timer::new(Duration::from_micros(ACCEL_FREQ));
                    rocket.flags.new_accel = Timer::new(Duration::from_millis(ALT_FREQ));

                    
                },
                Status::WaitForPeak => {
                    //Set the launch MET
                    rocket.start_met = Some(Instant::now());
                },
                Status::WaitForLanding => {
                    //Fire Solenoid
                    rocket.perf.solenoid.set_high();

                    //Take photo
                    use std::fs::File;
                    use std::io::Write;

                    let local: DateTime<Local> = Local::now();
                    let local = local.format("%Y-%m-%d_%H:%M:%S").to_string();
                    let filename = format!("{}.jpg",local);
                    let b = camera.take_one().expect("Camera 4");
                    File::create(filename.as_str()).expect("Camera 5").write_all(&b).expect("Camera 6");

                    //Dump to file
                    let mut file = OpenOptions::new().append(true).create(true).open(temp_file.as_str())?;
                    for entry in &rocket.log {
                        writeln!(file,"{}",entry)?;
                    }

                }
            }
            rocket.set_led()?;
            rocket.flags.status_change = false;

        }

        rocket.doing()?;
        rocket.transition()?;


        thread::sleep(Duration::from_micros(100));
    }

    #[allow(unreachable_code)]
    Ok(())
}
