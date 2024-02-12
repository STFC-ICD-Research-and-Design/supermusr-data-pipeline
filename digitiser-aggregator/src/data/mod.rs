mod event;
pub(crate) use event::EventData;

use supermusr_common::DigitizerId;

pub(crate) type DigitiserData<T> = Vec<(DigitizerId, T)>;

pub(crate) trait Accumulate<D> {
    fn accumulate(data: &mut DigitiserData<D>) -> D;
}
