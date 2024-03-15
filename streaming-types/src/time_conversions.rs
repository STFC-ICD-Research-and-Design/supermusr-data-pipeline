use anyhow::{anyhow, Result, Error};
use crate::frame_metadata_v1_generated::GpsTime;
use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc};

impl TryFrom<GpsTime> for DateTime<Utc> {
    fn try_from(t: GpsTime) -> Result<Self> {
        if t.nanosecond() > 999 {
            return Err(anyhow!("Timestamp Error ns = {0} > 999", t.nanosecond()))
        }
        if t.microsecond() > 999 {
            return Err(anyhow!("Timestamp Error: us = {0}", t.microsecond()))
        }
        if t.millisecond() > 999 {
            return Err(anyhow!("Timestamp Error: ms = {0}", t.millisecond()))
        }
        let nanosecond = (t.millisecond() as u32 * 1_000_000)
            + (t.microsecond() as u32 * 1_000)
            + (t.nanosecond() as u32);

        let dt = match NaiveDate::from_yo_opt(2000 + (t.year() as i32), t.day().into()) {
            Some(dt) => Ok(dt),
            None => Err(anyhow!("Timestamp Error: year: {0}, day: {1}", t.year(), t.day()))
        }?;
        let dt = match dt.and_hms_nano_opt(
                t.hour().into(),
                t.minute().into(),
                t.second().into(),
                nanosecond,
            ) {
            Some(dt) => Ok(dt),
            None => Err(anyhow!("Timestamp Error: hour: {0}, min: {1}, sec: {2}, nano {3}", t.hour(), t.minute(), t.second(), nanosecond))
            }?;
        match dt.and_local_timezone(Utc) {
            chrono::LocalResult::None => Err(anyhow!("Timezone cannot be added")),
            chrono::LocalResult::Single(dt) => Ok(dt),
            chrono::LocalResult::Ambiguous(_, _) => Err(anyhow!("Timezone ambiguous")),
        }
    }
    
    type Error = Error;
}

impl From<DateTime<Utc>> for GpsTime {
    fn from(t: DateTime<Utc>) -> Self {
        Self::new(
            (t.year() - 2000) as u8,
            t.ordinal() as u16,
            t.hour() as u8,
            t.minute() as u8,
            t.second() as u8,
            (t.nanosecond() / 1_000_000) as u16,
            ((t.nanosecond() % 1_000_000) / 1_000) as u16,
            (t.nanosecond() % 1_000) as u16,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpstime_to_datetimeutc() {
        let t1 = GpsTime::new(22, 205, 14, 52, 22, 100, 200, 300);

        let t2: DateTime<Utc> = t1.try_into().unwrap();

        assert_eq!(t2.year(), 2022);
        assert_eq!(t2.month(), 7);
        assert_eq!(t2.day(), 24);

        assert_eq!(t2.hour(), 14);
        assert_eq!(t2.minute(), 52);
        assert_eq!(t2.second(), 22);

        assert_eq!(t2.nanosecond(), 100200300);
    }

    #[test]
    fn datetimeutc_to_gpstime() {
        let t1 = NaiveDate::from_ymd_opt(2022, 7, 24)
            .unwrap()
            .and_hms_nano_opt(14, 52, 22, 100200300)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap();

        let t2: GpsTime = t1.into();

        assert_eq!(t2, GpsTime::new(22, 205, 14, 52, 22, 100, 200, 300));
    }
}
