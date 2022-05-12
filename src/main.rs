#![no_std]
#![no_main]

use arduino_hal::hal::wdt;
use embedded_hal::serial::Read;

mod control;
mod ds1307;
mod sr04;
mod time;

use control::ControlMode;
use control::Job;
use control::ManualControl;
use control::{Control, VentilGruppe, Waterbreach};
use ds1307::Ds1307;
use sr04::SR04;
use time::{Date, DateTime, Duration, Time};

#[arduino_hal::entry]
fn main() -> ! {
    // initialize Peripherals
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 9600);
    let i2c = arduino_hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        100,
    );
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    let wasser_pin = pins.a0.into_analog_input(&mut adc);

    // start watchdog
    let mut watchdog = wdt::Wdt::new(dp.WDT, &dp.CPU.mcusr);
    watchdog.start(wdt::Timeout::Ms8000).unwrap();

    // initialize sr04 and external clock
    let mut sr04 = SR04::new(dp.TC1, pins.d8.into_output(), pins.d9.forget_imode());
    let mut rtc = Ds1307::new(i2c);

    // make sure clock is running
    rtc.start().unwrap_or_else(|_| panic!());

    // get time and init control struct
    let starttime = rtc.get_datetime().unwrap_or_else(|_| panic!());

    let mut control = Control {
        start_time: starttime,
        current_time: starttime,
        ventil_gruppe: VentilGruppe::new(
            pins.d4.into_output(),
            pins.d5.into_output(),
            pins.d6.into_output(),
            pins.d7.into_output(),
        ),
        control_mode: ControlMode::Automatic(Job::Idle, Job::Idle),
        already_cleaned: false,
        distance: None,
        water_breach: Waterbreach(None),
    };

    let mut led = pins.d13.into_output();

    // main loop
    loop {
        led.toggle();

        if control.control_mode != ControlMode::Breach {
            control.distance = sr04.measure_distance();
        }

        control.current_time = rtc.get_datetime().unwrap_or_else(|_| panic!());

        // check for waterbreach
        if control.water_breach.0.is_none() && wasser_pin.analog_read(&mut adc) > 60 {
            control.water_breach = Waterbreach(Some(control.current_time));
            control.control_mode = ControlMode::Breach;
        }

        if let Ok(b) = serial.read() {
            match b as char {
                'a' => control.control_mode = ControlMode::Automatic(Job::Idle, Job::Idle),
                'b' => {
                    control.control_mode = ControlMode::Manual(ManualControl::CurrentJob(Job::Idle))
                }
                'c' => {
                    control.control_mode =
                        ControlMode::Manual(ManualControl::CurrentJob(Job::Filter))
                }
                'd' => {
                    control.control_mode = ControlMode::Manual(ManualControl::CurrentJob(
                        Job::Clean(Date::from_ymd(0, 0, 0).with_hms(0, 0, 0)),
                    ))
                }
                '1' => {
                    control.control_mode =
                        ControlMode::Manual(ManualControl::Bridged(true, false, false, false))
                }
                '2' => {
                    control.control_mode =
                        ControlMode::Manual(ManualControl::Bridged(false, true, false, false))
                }
                '3' => {
                    control.control_mode =
                        ControlMode::Manual(ManualControl::Bridged(false, false, true, false))
                }
                '4' => {
                    control.control_mode =
                        ControlMode::Manual(ManualControl::Bridged(false, false, false, true))
                }
                'o' => control.control_mode = ControlMode::Off,
                'r' => control.water_breach = Waterbreach(None),
                'p' => panic!(),
                _ => (),
            }
        }

        match control.control_mode {
            ControlMode::Automatic(ref job, ref next_job) => match (job, next_job) {
                (Job::Idle, _) => {
                    control.ventil_gruppe.set_idle();
                    if control.needs_cleaning(control.current_time.time) {
                        control.control_mode = ControlMode::Automatic(
                            Job::Clean(control.current_time.add_duration(Duration(10))),
                            Job::Idle,
                        );
                        control.already_cleaned = true;
                    } else if let Some(d) = control.distance {
                        if d > 50 {
                            control.control_mode = ControlMode::Automatic(
                                Job::Clean(control.current_time.add_duration(Duration(5))),
                                Job::Filter,
                            );
                        }
                    }
                }
                (Job::Filter, _) => {
                    control.ventil_gruppe.set_filter();
                    if control.distance.filter(|d| *d >= 10).is_none() {
                        control.control_mode = ControlMode::Automatic(
                            Job::Clean(control.current_time.add_duration(Duration(5))),
                            Job::Idle,
                        );
                    }
                }
                (Job::Clean(stoptime), Job::Idle) => {
                    control.ventil_gruppe.set_clean();
                    if control.current_time.ge(stoptime) {
                        control.control_mode = ControlMode::Automatic(Job::Idle, Job::Idle);
                    }
                }
                (Job::Clean(stoptime), Job::Filter) => {
                    control.ventil_gruppe.set_clean();
                    if control.current_time.ge(stoptime) {
                        control.control_mode = ControlMode::Automatic(Job::Filter, Job::Idle);
                    }
                }
                _ => {
                    control.ventil_gruppe.set_idle();
                }
            },
            ControlMode::Manual(ref manualcontrol) => match manualcontrol {
                ManualControl::CurrentJob(job) => match job {
                    Job::Idle => {
                        control.ventil_gruppe.set_idle();
                    }
                    Job::Filter => {
                        control.ventil_gruppe.set_filter();
                    }
                    Job::Clean(_) => {
                        control.ventil_gruppe.set_clean();
                    }
                },
                ManualControl::Bridged(v1, v2, v3, v4) => {
                    control.ventil_gruppe.einlass.set(*v1);
                    control.ventil_gruppe.abwasser.set(*v2);
                    control.ventil_gruppe.filterwasser.set(*v3);
                    control.ventil_gruppe.bridge.set(*v4);
                }
            },
            ControlMode::Breach => control.ventil_gruppe.set_idle(),
            ControlMode::Off => {
                control.ventil_gruppe.set_idle();
            }
        }

        ufmt::uwriteln!(&mut serial, "{:?}!!", control).unwrap();

        watchdog.feed();
        arduino_hal::delay_ms(4000);
    }
}

// the watchdog should restart the device afer a panic
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // disable interrupts - firmware has panicked so no ISRs should continue running
    avr_device::interrupt::disable();

    // get the peripherals so we can access the LED.
    //
    // SAFETY: Because main() already has references to the peripherals this is an unsafe
    // operation - but because no other code can run after the panic handler was called,
    // we know it is okay.
    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let pins = arduino_hal::pins!(dp);

    // Blink LED rapidly
    let mut led = pins.d13.into_output();
    loop {
        led.toggle();
        arduino_hal::delay_ms(50);
    }
}
