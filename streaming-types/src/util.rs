use crate::status_packet_v1_generated::GpsTime;
use chrono::{NaiveDate, NaiveDateTime};

impl From<GpsTime> for NaiveDateTime {
    fn from(t: GpsTime) -> Self {
        let nanosecond = (t.microsecond() as u32 * 1_000_000)
            + (t.millisecond() as u32 * 1_000)
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
