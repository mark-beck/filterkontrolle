#![allow(dead_code)]

use ufmt::{uDebug, uDisplay, uWrite, uwrite};

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct Time {
    hour: u16,
    minute: u16,
    second: u16,
}

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct Date {
    year: u16,
    month: u16,
    day: u16,
}

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct DateTime {
    pub date: Date,
    pub time: Time,
}

impl Time {
    pub fn from_hms(hour: u16, minute: u16, second: u16) -> Self {
        Self {
            hour,
            minute,
            second,
        }
    }

    pub fn hour(&self) -> u16 {
        self.hour
    }
    pub fn minute(&self) -> u16 {
        self.minute
    }
    pub fn second(&self) -> u16 {
        self.second
    }

    pub fn since(&self, othertime: &Time) -> Duration {
        Duration(
            ((self.hour() as i16 - othertime.hour() as i16) * 60 * 60)
                + ((self.minute() as i16 - othertime.minute() as i16) * 60)
                + (self.second() as i16 - othertime.second() as i16),
        )
    }

    pub fn add_duration(mut self, mut dur: Duration) -> Self {
        while dur.0 > 0 {
            if self.second == 60 {
                self.second = 0;
                if self.minute == 60 {
                    self.minute = 0;
                    if self.hour == 24 {
                        self.hour = 0;
                    } else {
                        self.hour += 1;
                    }
                } else {
                    self.minute += 1;
                }
            } else {
                self.second += 1;
            }
            dur.0 -= 1;
        }
        self
    }
}

impl Date {
    pub fn get_days_in_month(month: u16) -> u16 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            2 => 28,
            _ => 30,
        }
    }

    pub fn from_ymd(year: u16, month: u16, day: u16) -> Self {
        Self { year, month, day }
    }

    pub fn year(&self) -> u16 {
        self.year
    }
    pub fn month(&self) -> u16 {
        self.month
    }
    pub fn day(&self) -> u16 {
        self.day
    }
    pub fn with_hms(self, hour: u16, minute: u16, second: u16) -> DateTime {
        DateTime {
            date: self,
            time: Time::from_hms(hour, minute, second),
        }
    }
}

impl DateTime {
    pub fn add_duration(mut self, mut dur: Duration) -> Self {
        while dur.0 > 0 {
            if self.time.second == 60 {
                self.time.second = 0;
                if self.time.minute == 60 {
                    self.time.minute = 0;
                    if self.time.hour == 24 {
                        self.time.hour = 0;
                        if self.date.day == Date::get_days_in_month(self.date.month) {
                            self.date.day = 1;
                            if self.date.month == 12 {
                                self.date.month = 1;
                                self.date.year += 1;
                            } else {
                                self.date.month += 1;
                            }
                        } else {
                            self.date.day += 1;
                        }
                    } else {
                        self.time.hour += 1;
                    }
                } else {
                    self.time.minute += 1;
                }
            } else {
                self.time.second += 1;
            }
            dur.0 -= 1;
        }
        self
    }
}

impl uDisplay for Time {
    fn fmt<W: ?Sized>(&self, f: &mut ufmt::Formatter<W>) -> Result<(), W::Error>
    where
        W: uWrite,
    {
        uwrite!(f, "{}:{}:{}", self.hour, self.minute, self.second)
    }
}

impl uDebug for Time {
    fn fmt<W: ?Sized>(&self, f: &mut ufmt::Formatter<W>) -> Result<(), W::Error>
    where
        W: uWrite,
    {
        uwrite!(f, "{}:{}:{}", self.hour, self.minute, self.second)
    }
}

impl uDisplay for Date {
    fn fmt<W: ?Sized>(&self, f: &mut ufmt::Formatter<W>) -> Result<(), W::Error>
    where
        W: uWrite,
    {
        uwrite!(f, "{}:{}:{}", self.year, self.month, self.day)
    }
}

impl uDebug for Date {
    fn fmt<W: ?Sized>(&self, f: &mut ufmt::Formatter<W>) -> Result<(), W::Error>
    where
        W: uWrite,
    {
        uwrite!(f, "{}:{}:{}", self.year, self.month, self.day)
    }
}

impl uDisplay for DateTime {
    fn fmt<W: ?Sized>(&self, f: &mut ufmt::Formatter<W>) -> Result<(), W::Error>
    where
        W: uWrite,
    {
        uwrite!(f, "{}:{}", self.date, self.time)
    }
}

impl uDebug for DateTime {
    fn fmt<W: ?Sized>(&self, f: &mut ufmt::Formatter<W>) -> Result<(), W::Error>
    where
        W: uWrite,
    {
        uwrite!(f, "{}:{}", self.date, self.time)
    }
}

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub struct Duration(pub i16);

impl Duration {
    pub fn seconds(&self) -> i16 {
        self.0
    }
    pub fn minutes(&self) -> i16 {
        self.0 / 60
    }
    pub fn hours(&self) -> i16 {
        self.0 / 3600
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_adding() {
        assert_eq!(
            Time::from_hms(1, 30, 30),
            Time::from_hms(0, 30, 30).add_duration(Duration(3600))
        );
    }
}
