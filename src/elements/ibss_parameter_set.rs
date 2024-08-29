use core::time::Duration;

use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::common::TU;

use super::{Element, ElementID};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// The IBSS Parameter Set element contains the set of parameters necessary to support an IBSS.
pub struct IBSSParameterSetElement {
    /// The ATIM window length in [crate::common::TU]s.
    /// Use the [Self::atim_window_in_tu] function to get the duration.
    pub atim_window: u16,
}
impl IBSSParameterSetElement {
    /// Returns a [Duration] for the ATIM window length.
    pub const fn atim_window_in_tu(&self) -> Duration {
        Duration::from_micros(TU.as_micros() as u64 * self.atim_window as u64)
    }
    /// Create a new [IBSSParameterSetElement] from a [Duration].
    pub const fn new(atim_window_duration: Duration) -> Self {
        Self {
            atim_window: (atim_window_duration.as_micros() / TU.as_micros()) as u16,
        }
    }
}
impl MeasureWith<()> for IBSSParameterSetElement {
    fn measure_with(&self, _ctx: &()) -> usize {
        2
    }
}
impl TryFromCtx<'_> for IBSSParameterSetElement {
    type Error = scroll::Error;
    fn try_from_ctx(from: &[u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        Ok((
            Self {
                atim_window: from.pread_with(0, Endian::Little)?,
            },
            2,
        ))
    }
}
impl TryIntoCtx for IBSSParameterSetElement {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        buf.pwrite_with(self.atim_window, 0, Endian::Little)
    }
}

impl Element for IBSSParameterSetElement {
    const ELEMENT_ID: ElementID = ElementID::Id(0x06);
    type ReadType<'a> = Self;
}
