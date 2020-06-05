use std::convert::TryInto;
use std::fmt;
use std::num::TryFromIntError;

use chrono::{NaiveTime, Timelike};

// TODO: rename as DateTime and take Month enum?

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExtendedTime {
    hour: u8,
    minute: u8,
}

impl fmt::Debug for ExtendedTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        write!(f, "{}:{:02}", self.hour, self.minute)
    }
}

impl ExtendedTime {
    pub fn new(hour: u8, minute: u8) -> Self {
        if minute >= 60 {
            panic!("invalid time: minute is {}", minute)
        }

        Self { hour, minute }
    }

    pub fn hour(self) -> u8 {
        self.hour
    }

    pub fn minute(self) -> u8 {
        self.minute
    }

    pub fn add_minutes(&self, minutes: i16) -> Result<Self, TryFromIntError> {
        let as_minutes = self.mins_from_midnight() as i16 + minutes;
        Ok(Self::from_mins_from_midnight(as_minutes.try_into()?))
    }

    pub fn add_hours(&self, hours: i16) -> Result<Self, TryFromIntError> {
        Ok(Self {
            hour: (i16::from(self.hour) + hours).try_into()?,
            minute: self.minute,
        })
    }

    pub fn from_mins_from_midnight(minute: u16) -> Self {
        let hour = (minute / 60).try_into().unwrap();
        let minute = (minute % 60).try_into().expect("time from minute overflow");
        Self { hour, minute }
    }

    pub fn mins_from_midnight(self) -> u16 {
        u16::from(self.minute) + 60 * u16::from(self.hour)
    }
}

impl TryInto<NaiveTime> for ExtendedTime {
    type Error = ();

    fn try_into(self) -> Result<NaiveTime, Self::Error> {
        NaiveTime::from_hms_opt(self.hour.into(), self.minute.into(), 0).ok_or(())
    }
}

impl From<NaiveTime> for ExtendedTime {
    fn from(time: NaiveTime) -> ExtendedTime {
        Self {
            hour: time.hour().try_into().expect("invalid NaiveTime"),
            minute: time.minute().try_into().expect("invalid NaiveTime"),
        }
    }
}
