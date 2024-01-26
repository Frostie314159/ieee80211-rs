use scroll::{
    ctx::{MeasureWith, SizeWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use super::{ToTLV, IEEE80211TLV};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// TLV containing the current channel of the sender.
pub struct DSSSParameterSet {
    pub current_channel: u8,
}
impl SizeWith for DSSSParameterSet {
    fn size_with(_ctx: &()) -> usize {
        1
    }
}
impl MeasureWith<()> for DSSSParameterSet {
    fn measure_with(&self, ctx: &()) -> usize {
        Self::size_with(ctx)
    }
}
impl TryFromCtx<'_> for DSSSParameterSet {
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
impl TryIntoCtx for DSSSParameterSet {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        buf.pwrite(self.current_channel, 0)
    }
}
impl<'a> ToTLV<'a> for DSSSParameterSet {
    fn to_tlv(self) -> IEEE80211TLV<'a> {
        IEEE80211TLV::DSSSParameterSet(self)
    }
}
