use std::convert::{TryFrom, TryInto};
use std::fmt::Display;
use std::ops::RangeInclusive;

use chrono::prelude::Datelike;
use chrono::{Duration, NaiveDate};

// Reexport Weekday from chrono as part of the public type.
pub use chrono::Weekday;

// Display

fn wday_str(wday: Weekday) -> &'static str {
    match wday {
        Weekday::Mon => "Mo",
        Weekday::Tue => "Tu",
        Weekday::Wed => "We",
        Weekday::Thu => "Th",
        Weekday::Fri => "Fr",
        Weekday::Sat => "Sa",
        Weekday::Sun => "Su",
    }
}

fn write_days_offset(f: &mut std::fmt::Formatter<'_>, offset: i64) -> std::fmt::Result {
    if offset == 0 {
        return Ok(());
    }

    write!(f, " ")?;

    if offset > 0 {
        write!(f, "+")?;
    }

    write!(f, "{offset} day")?;

    if offset.abs() > 1 {
        write!(f, "s")?;
    }

    Ok(())
}

// Errors

#[derive(Debug)]
pub struct InvalidMonth;

// DaySelector

#[derive(Clone, Debug, Default)]
pub struct DaySelector {
    pub year: Vec<YearRange>,
    pub monthday: Vec<MonthdayRange>,
    pub week: Vec<WeekRange>,
    pub weekday: Vec<WeekDayRange>,
}

// YearRange

#[derive(Clone, Debug)]
pub struct YearRange {
    pub range: RangeInclusive<u16>,
    pub step: u16,
}

// MonthdayRange

#[derive(Clone, Debug)]
pub enum MonthdayRange {
    Month {
        range: RangeInclusive<Month>,
        year: Option<u16>,
    },
    Date {
        start: (Date, DateOffset),
        end: (Date, DateOffset),
    },
}

// Date

#[derive(Clone, Copy, Debug)]
pub enum Date {
    Fixed {
        year: Option<u16>,
        month: Month,
        day: u8,
    },
    Easter {
        year: Option<u16>,
    },
}

impl Date {
    #[inline]
    pub fn day(day: u8, month: Month, year: u16) -> Self {
        Self::Fixed { day, month, year: Some(year) }
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Date::Fixed { year, month, day } => {
                if let Some(year) = year {
                    write!(f, "{year} ")?;
                }

                write!(f, "{month} {day}")?;
            }
            Date::Easter { year } => {
                if let Some(year) = year {
                    write!(f, "{year} ")?;
                }

                write!(f, "easter")?;
            }
        }

        Ok(())
    }
}

// DateOffset

#[derive(Clone, Debug, Default)]
pub struct DateOffset {
    pub wday_offset: WeekDayOffset,
    pub day_offset: i64,
}

impl DateOffset {
    #[inline]
    pub fn apply(&self, mut date: NaiveDate) -> NaiveDate {
        date += Duration::days(self.day_offset);

        match self.wday_offset {
            WeekDayOffset::None => {}
            WeekDayOffset::Prev(target) => {
                let diff = (7 + target as i64 - date.weekday() as i64) % 7;
                date -= Duration::days(diff)
            }
            WeekDayOffset::Next(target) => {
                let diff = (7 + date.weekday() as i64 - target as i64) % 7;
                date += Duration::days(diff)
            }
        }

        date
    }
}

impl Display for DateOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.wday_offset)?;
        write_days_offset(f, self.day_offset)?;
        Ok(())
    }
}

// WeekDayOffset

#[derive(Clone, Copy, Debug)]
pub enum WeekDayOffset {
    None,
    Next(Weekday),
    Prev(Weekday),
}

impl Default for WeekDayOffset {
    #[inline]
    fn default() -> Self {
        Self::None
    }
}

impl Display for WeekDayOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => {}
            Self::Next(wday) => write!(f, "+{}", wday_str(*wday))?,
            Self::Prev(wday) => write!(f, "-{}", wday_str(*wday))?,
        }

        Ok(())
    }
}

// WeekDayRange

#[derive(Clone, Debug)]
pub enum WeekDayRange {
    Fixed {
        range: RangeInclusive<Weekday>,
        offset: i64,
        nth: [bool; 5],
    },
    Holiday {
        kind: HolidayKind,
        offset: i64,
    },
}

impl Display for WeekDayRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fixed { range, offset, nth } => {
                write!(f, "{}", wday_str(*range.start()))?;

                if range.start() != range.end() {
                    write!(f, "-{}", wday_str(*range.end()))?;
                }

                if nth.contains(&true) {
                    let mut weeknum_iter = nth
                        .iter()
                        .enumerate()
                        .filter(|(_, x)| **x)
                        .map(|(idx, _)| idx + 1);

                    write!(f, "[{}", weeknum_iter.next().unwrap())?;

                    for num in weeknum_iter {
                        write!(f, ",{num}")?;
                    }

                    write!(f, "]")?;
                }

                write_days_offset(f, *offset)?;
            }
            Self::Holiday { kind, offset } => {
                write!(f, "{kind}")?;

                if *offset != 0 {
                    write!(f, " {offset}")?;
                }
            }
        }

        Ok(())
    }
}

// HolidayKind

#[derive(Clone, Copy, Debug)]
pub enum HolidayKind {
    Public,
    School,
}

impl Display for HolidayKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => write!(f, "PH"),
            Self::School => write!(f, "SH"),
        }
    }
}

// WeekRange

#[derive(Clone, Debug)]
pub struct WeekRange {
    pub range: RangeInclusive<u8>,
    pub step: u8,
}

impl Display for WeekRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.range.start())?;

        if self.range.start() != self.range.end() {
            write!(f, "-{}", self.range.end())?;
        }

        if self.step != 1 {
            write!(f, "/{}", self.step)?;
        }

        Ok(())
    }
}

// Month

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Month {
    January = 1,
    February = 2,
    March = 3,
    April = 4,
    May = 5,
    June = 6,
    July = 7,
    August = 8,
    September = 9,
    October = 10,
    November = 11,
    December = 12,
}

impl Month {
    #[inline]
    pub fn next(self) -> Self {
        let num = self as u8;
        ((num % 12) + 1).try_into().unwrap()
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Month::January => "January",
            Month::February => "February",
            Month::March => "March",
            Month::April => "April",
            Month::May => "May",
            Month::June => "June",
            Month::July => "July",
            Month::August => "August",
            Month::September => "September",
            Month::October => "October",
            Month::November => "November",
            Month::December => "December",
        }
    }
}

impl Display for Month {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.as_str()[..3])
    }
}

macro_rules! impl_try_into_for_month {
    ( $from_type: ty ) => {
        impl TryFrom<$from_type> for Month {
            type Error = InvalidMonth;

            #[inline]
            fn try_from(value: $from_type) -> Result<Self, Self::Error> {
                let value: u8 = value.try_into().map_err(|_| InvalidMonth)?;

                Ok(match value {
                    1 => Self::January,
                    2 => Self::February,
                    3 => Self::March,
                    4 => Self::April,
                    5 => Self::May,
                    6 => Self::June,
                    7 => Self::July,
                    8 => Self::August,
                    9 => Self::September,
                    10 => Self::October,
                    11 => Self::November,
                    12 => Self::December,
                    _ => return Err(InvalidMonth),
                })
            }
        }
    };
    ( $from_type: ty, $( $tail: tt )+ ) => {
        impl_try_into_for_month!($from_type);
        impl_try_into_for_month!($($tail)+);
    };
}

impl_try_into_for_month!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize);
