use scroll::{
    ctx::{MeasureWith, SizeWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use super::{Element, ElementID};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// The DSSS Parameter Set element contains information to allow channel number identification for STAs.
pub struct DSSSParameterSetElement {
    pub current_channel: u8,
}
impl SizeWith for DSSSParameterSetElement {
    fn size_with(_ctx: &()) -> usize {
        1
    }
}
impl MeasureWith<()> for DSSSParameterSetElement {
    fn measure_with(&self, ctx: &()) -> usize {
        Self::size_with(ctx)
    }
}
impl TryFromCtx<'_> for DSSSParameterSetElement {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'_ [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        Ok((
            Self {
                current_channel: from.pread(0)?,
            },
            1,
        ))
    }
}
impl TryIntoCtx for DSSSParameterSetElement {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        buf.pwrite(self.current_channel, 0)
    }
}

impl Element for DSSSParameterSetElement {
    const ELEMENT_ID: ElementID = ElementID::Id(0x03);
    type ReadType<'a> = Self;
}
