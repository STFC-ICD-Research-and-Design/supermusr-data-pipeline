use crate::status_packet_v1_generated::GpsTime;
use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};

impl From<GpsTime> for NaiveDateTime {
    fn from(t: GpsTime) -> Self {
        let nanosecond = (t.millisecond() as u32 * 1_000_000)
            + (t.microsecond() as u32 * 1_000)
            + (t.nanosecond() as u32);

        NaiveDate::from_ymd(2000 + (t.year() as i32), t.month().into(), t.day().into())
            .and_hms_nano(
                t.hour().into(),
                t.minute().into(),
                t.second().into(),
                nanosecond,
            )
    }
}

impl From<NaiveDateTime> for GpsTime {
    fn from(t: NaiveDateTime) -> Self {
        Self::new(
            (t.year() - 2000) as u8,
            t.month() as u8,
            t.day() as u8,
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
    fn test_gpstime_to_naivedatetime() {
        let t1 = GpsTime::new(22, 07, 24, 14, 52, 22, 100, 200, 300);

        let t2: NaiveDateTime = t1.into();

        assert_eq!(t2.year(), 2022);
        assert_eq!(t2.month(), 07);
        assert_eq!(t2.day(), 24);

        assert_eq!(t2.hour(), 14);
        assert_eq!(t2.minute(), 52);
        assert_eq!(t2.second(), 22);

        assert_eq!(t2.nanosecond(), 100200300);
    }

    #[test]
    fn test_naivedatetime_to_gpstime() {
        let t1 = NaiveDate::from_ymd(2022, 07, 24).and_hms_nano(14, 52, 22, 100200300);

        let t2: GpsTime = t1.into();

        assert_eq!(t2, GpsTime::new(22, 07, 24, 14, 52, 22, 100, 200, 300));
    }
}
