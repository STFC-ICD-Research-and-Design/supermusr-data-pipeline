use crate::frame_metadata_v1_generated::GpsTime;
use chrono::{DateTime, Datelike, LocalResult, NaiveDate, Timelike, Utc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GpsTimeConversionError {
    #[error("GpsTime Component(s) Out of Range: {0:?}")]
    OutOfRangeComponent(GpsTime),
    #[error("GpsTime Timezone Error: {0:?}")]
    Timezone(LocalResult<DateTime<Utc>>),
}

impl TryFrom<GpsTime> for DateTime<Utc> {
    type Error = GpsTimeConversionError;

    fn try_from(t: GpsTime) -> Result<Self, Self::Error> {
        if t.nanosecond() > 999 || t.microsecond() > 999 || t.millisecond() > 999 {
            return Err(GpsTimeConversionError::OutOfRangeComponent(t));
        }

        let nanosecond = (t.millisecond() as u32 * 1_000_000)
            + (t.microsecond() as u32 * 1_000)
            + (t.nanosecond() as u32);

        NaiveDate::from_yo_opt(2000 + (t.year() as i32), t.day().into())
            .ok_or(GpsTimeConversionError::OutOfRangeComponent(t))
            .and_then(|dt| {
                dt.and_hms_nano_opt(
                    t.hour().into(),
                    t.minute().into(),
                    t.second().into(),
                    nanosecond,
                )
                .ok_or(GpsTimeConversionError::OutOfRangeComponent(t))
            })
            .and_then(|dt| match dt.and_local_timezone(Utc) {
                LocalResult::Single(dt) => Ok(dt),
                other => Err(GpsTimeConversionError::Timezone(other)),
            })
    }
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

    #[test]
    fn gpstime_invalid_day() {
        let t1 = GpsTime::new(22, 366, 14, 52, 22, 100, 200, 300);
        let t2: Result<DateTime<Utc>, _> = t1.try_into();
        assert!(t2.is_err());
        let err = t2.unwrap_err();
        assert_eq!(
            format!("{err}"),
            format!("GpsTime Component(s) Out of Range: {0:?}", t1)
        );
    }

    #[test]
    fn gpstime_invalid_hour() {
        let t1 = GpsTime::new(22, 205, 24, 52, 22, 100, 200, 300);
        let t2: Result<DateTime<Utc>, _> = t1.try_into();
        assert!(t2.is_err());
        let err = t2.unwrap_err();
        assert_eq!(
            format!("{err}"),
            format!("GpsTime Component(s) Out of Range: {0:?}", t1)
        );
    }

    #[test]
    fn gpstime_invalid_minute() {
        let t1 = GpsTime::new(22, 205, 23, 80, 22, 100, 200, 300);
        let t2: Result<DateTime<Utc>, _> = t1.try_into();
        assert!(t2.is_err());
        let err = t2.unwrap_err();
        assert_eq!(
            format!("{err}"),
            format!("GpsTime Component(s) Out of Range: {0:?}", t1)
        );
    }

    #[test]
    fn gpstime_invalid_second() {
        let t1 = GpsTime::new(22, 205, 23, 22, 80, 100, 200, 300);
        let t2: Result<DateTime<Utc>, _> = t1.try_into();
        assert!(t2.is_err());
        let err = t2.unwrap_err();
        assert_eq!(
            format!("{err}"),
            format!("GpsTime Component(s) Out of Range: {0:?}", t1)
        );
    }

    #[test]
    fn gpstime_invalid_millisecond() {
        let t1 = GpsTime::new(22, 205, 23, 22, 4, 1000, 200, 300);
        let t2: Result<DateTime<Utc>, _> = t1.try_into();
        assert!(t2.is_err());
        let err = t2.unwrap_err();
        assert_eq!(
            format!("{err}"),
            format!("GpsTime Component(s) Out of Range: {0:?}", t1)
        );
    }

    #[test]
    fn gpstime_invalid_microsecond() {
        let t1 = GpsTime::new(22, 205, 23, 22, 4, 200, 1000, 300);
        let t2: Result<DateTime<Utc>, _> = t1.try_into();
        assert!(t2.is_err());
        let err = t2.unwrap_err();
        assert_eq!(
            format!("{err}"),
            format!("GpsTime Component(s) Out of Range: {0:?}", t1)
        );
    }

    #[test]
    fn gpstime_invalid_nanosecond() {
        let t1 = GpsTime::new(22, 205, 23, 22, 4, 200, 300, 1000);
        let t2: Result<DateTime<Utc>, _> = t1.try_into();
        assert!(t2.is_err());
        let err = t2.unwrap_err();
        assert_eq!(
            format!("{err}"),
            format!("GpsTime Component(s) Out of Range: {0:?}", t1)
        );
    }
}
