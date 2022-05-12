#![allow(dead_code)]

use crate::time::{Date, DateTime, Time};
use embedded_hal::blocking::i2c::{Write, WriteRead};

pub struct Register;
impl Register {
    pub const SECONDS: u8 = 0x00;
    pub const MINUTES: u8 = 0x01;
    pub const HOURS: u8 = 0x02;
    pub const DOW: u8 = 0x03;
    pub const DOM: u8 = 0x04;
    pub const MONTH: u8 = 0x05;
    pub const YEAR: u8 = 0x06;
    pub const SQWOUT: u8 = 0x07;
    pub const RAM_BEGIN: u8 = 0x08;
    pub const RAM_END: u8 = 0x3F;
}

pub struct BitFlags;
impl BitFlags {
    pub const H24_H12: u8 = 0b0100_0000;
    pub const AM_PM: u8 = 0b0010_0000;
    pub const CH: u8 = 0b1000_0000;
    pub const SQWE: u8 = 0b0001_0000;
    pub const OUTLEVEL: u8 = 0b1000_0000;
    pub const OUTRATERS0: u8 = 0b0000_0001;
    pub const OUTRATERS1: u8 = 0b0000_0010;
}

pub const ADDR: u8 = 0b110_1000;

pub enum Error {
    I2C,
}

pub struct Ds1307<I2C> {
    i2c: I2C,
}

impl<I2C> Ds1307<I2C>
where
    I2C: Write + WriteRead,
{
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    pub fn destroy(self) -> I2C {
        self.i2c
    }

    pub fn is_running(&mut self) -> Result<bool, Error> {
        self.register_bit_flag_high(Register::SECONDS, BitFlags::CH)
    }

    /// Set the clock to run (default on power-on).
    /// (Does not alter the device register if already running).
    pub fn stop(&mut self) -> Result<(), Error> {
        self.set_register_bit_flag(Register::SECONDS, BitFlags::CH)
    }

    /// Halt the clock.
    /// (Does not alter the device register if already halted).
    pub fn start(&mut self) -> Result<(), Error> {
        self.clear_register_bit_flag(Register::SECONDS, BitFlags::CH)
    }

    pub fn get_seconds(&mut self) -> Result<u8, Error> {
        let data = self.read_register(Register::SECONDS)?;
        Ok(packed_bcd_to_decimal(data & !BitFlags::CH))
    }

    pub fn get_minutes(&mut self) -> Result<u8, Error> {
        let data = self.read_register(Register::MINUTES)?;
        Ok(packed_bcd_to_decimal(data))
    }

    pub fn get_hours(&mut self) -> Result<u8, Error> {
        let data = self.read_register(Register::HOURS)?;
        Ok(packed_bcd_to_decimal(data & !BitFlags::H24_H12))
    }

    fn get_day(&mut self) -> Result<u8, Error> {
        let data = self.read_register(Register::DOM)?;
        Ok(packed_bcd_to_decimal(data))
    }

    fn get_month(&mut self) -> Result<u8, Error> {
        let data = self.read_register(Register::MONTH)?;
        Ok(packed_bcd_to_decimal(data))
    }

    fn get_year(&mut self) -> Result<u16, Error> {
        let data = self.read_register(Register::YEAR)?;
        Ok(packed_bcd_to_decimal(data) as u16 + 2000)
    }

    pub fn get_time(&mut self) -> Result<Time, Error> {
        Ok(Time::from_hms(
            self.get_hours()? as u16,
            self.get_minutes()? as u16,
            self.get_seconds()? as u16,
        ))
    }

    pub fn get_date(&mut self) -> Result<Date, Error> {
        Ok(Date::from_ymd(
            self.get_year()?,
            self.get_month()? as u16,
            self.get_day()? as u16,
        ))
    }

    pub fn get_datetime(&mut self) -> Result<DateTime, Error> {
        Ok(DateTime {
            date: self.get_date()?,
            time: self.get_time()?,
        })
    }

    fn register_bit_flag_high(&mut self, address: u8, bitmask: u8) -> Result<bool, Error> {
        let data = self.read_register(address)?;
        Ok((data & bitmask) != 0)
    }

    fn set_register_bit_flag(&mut self, address: u8, bitmask: u8) -> Result<(), Error> {
        let data = self.read_register(address)?;
        if (data & bitmask) == 0 {
            self.write_register(address, data | bitmask)
        } else {
            Ok(())
        }
    }

    fn clear_register_bit_flag(&mut self, address: u8, bitmask: u8) -> Result<(), Error> {
        let data = self.read_register(address)?;
        if (data & bitmask) != 0 {
            self.write_register(address, data & !bitmask)
        } else {
            Ok(())
        }
    }

    fn write_register(&mut self, register: u8, data: u8) -> Result<(), Error> {
        let payload: [u8; 2] = [register, data];
        self.i2c.write(ADDR, &payload).map_err(|_| Error::I2C)
    }

    fn read_register(&mut self, register: u8) -> Result<u8, Error> {
        let mut data = [0];
        self.i2c
            .write_read(ADDR, &[register], &mut data)
            .map_err(|_| Error::I2C)
            .and(Ok(data[0]))
    }
}

fn packed_bcd_to_decimal(bcd: u8) -> u8 {
    (bcd >> 4) * 10 + (bcd & 0xF)
}
