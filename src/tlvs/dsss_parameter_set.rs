use scroll::{
    ctx::{TryFromCtx, TryIntoCtx, MeasureWith, SizeWith},
    Pread, Pwrite,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
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
