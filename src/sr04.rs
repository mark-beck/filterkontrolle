#![allow(dead_code)]

use arduino_hal::pac::TC1;
use arduino_hal::port::mode::{Input, Output};
use arduino_hal::port::Pin;

pub struct SR04<PINTRIG, PINECHO> {
    timer1: TC1,
    trig: Pin<Output, PINTRIG>,
    echo: Pin<Input, PINECHO>,
}

impl<PINTRIG, PINECHO> SR04<PINTRIG, PINECHO>
where
    PINTRIG: avr_hal_generic::port::PinOps,
    PINECHO: avr_hal_generic::port::PinOps,
{
    pub fn new(timer1: TC1, trig: Pin<Output, PINTRIG>, echo: Pin<Input, PINECHO>) -> Self {
        Self { timer1, trig, echo }
    }

    pub fn measure_distance(&mut self) -> Option<u16> {
        self.timer1.tccr1b.write(|w| w.cs1().prescale_64());

        // the timer is reinitialized with value 0.
        self.timer1.tcnt1.write(|w| unsafe { w.bits(0) });

        // the trigger must be set to high under 10 µs as per the HC-SR04 datasheet
        self.trig.set_high();
        arduino_hal::delay_us(10);
        self.trig.set_low();

        while self.echo.is_low() {
            // exiting the loop if the timer has reached 200 ms.
            // 0.2s/4µs = 50000
            if self.timer1.tcnt1.read().bits() >= 50000 {
                // jump to the beginning of the outer loop if no obstacle is detected
                return None;
            }
        }
        // Restarting the timer
        self.timer1.tcnt1.write(|w| unsafe { w.bits(0) });

        // Wait for the echo to get low again
        while self.echo.is_high() {}

        // 1 count == 4 µs, so the value is multiplied by 4.
        // 1/58 ≈ (34000 cm/s) * 1µs / 2
        // when no object is detected, instead of keeping the echo pin completely low,
        // some HC-SR04 labeled sensor holds the echo pin in high state for very long time,
        // thus overflowing the u16 value when multiplying the timer1 value with 4.
        // overflow during runtime causes panic! so it must be handled
        let temp_timer = self.timer1.tcnt1.read().bits().saturating_mul(4);
        let value = match temp_timer {
            u16::MAX => {
                return None;
            }
            _ => temp_timer / 58,
        };

        // Await 100 ms before sending the next trig
        // 0.1s/4µs = 25000
        while self.timer1.tcnt1.read().bits() < 25000 {}

        return Some(value);
    }
}
