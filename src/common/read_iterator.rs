use core::marker::PhantomData;

use scroll::ctx::TryFromCtx;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ReadIterator<'a, Ctx, Type> {
    pub(crate) bytes: Option<&'a [u8]>,
    _phantom: PhantomData<(Ctx, Type)>,
}
impl<'a, Ctx, Type> ReadIterator<'a, Ctx, Type> {
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: Some(bytes),
            _phantom: PhantomData,
        }
    }
}
impl<'a, Ctx: Default + Copy, Type: TryFromCtx<'a, Ctx, Error = scroll::Error>> Iterator
    for ReadIterator<'a, Ctx, Type>
{
    type Item = Type;
    fn next(&mut self) -> Option<Self::Item> {
        let bytes = self.bytes?;
        match Type::try_from_ctx(bytes, Ctx::default()) {
            Ok((ret, offset)) => {
                self.bytes = Some(&bytes[offset..]);
                Some(ret)
            }
            Err(_) => {
                self.bytes = None;
                None
            }
        }
    }
}
