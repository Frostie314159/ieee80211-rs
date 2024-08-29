use core::{fmt::Debug, marker::PhantomData};

use scroll::ctx::{MeasureWith, TryFromCtx};

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
/// An iterator recursively parsing data from a byte slice, until there's no more left.
pub struct ReadIterator<'a, Ctx, Type> {
    pub bytes: Option<&'a [u8]>,
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
impl<'a, Ctx: Default + Copy, Type: TryFromCtx<'a, Ctx, Error = scroll::Error> + Debug + Copy> Debug
    for ReadIterator<'a, Ctx, Type>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(*self).finish()
    }
}
#[cfg(feature = "defmt")]
impl<
        'a,
        Ctx: Default + Copy,
        Type: TryFromCtx<'a, Ctx, Error = scroll::Error> + defmt::Format + Copy,
    > defmt::Format for ReadIterator<'a, Ctx, Type>
{
    fn format(&self, fmt: defmt::Formatter) {
        let mut iter = self.clone();
        defmt::write!(fmt, "[");

        if let Some(first) = iter.next() {
            // We treat the first element differently, since the loop below prepends the comma to every element.
            defmt::write!(fmt, "{}", first);
            for next in iter {
                defmt::write!(fmt, ", {}", next);
            }
        }

        defmt::write!(fmt, "]");
    }
}
impl<'a, Ctx: Default + Copy, Type: TryFromCtx<'a, Ctx, Error = scroll::Error>> Iterator
    for ReadIterator<'a, Ctx, Type>
{
    type Item = Type;
    #[inline]
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
impl<
        'a,
        Ctx: Default + Copy,
        Type: MeasureWith<()> + TryFromCtx<'a, Ctx, Error = scroll::Error>,
    > ExactSizeIterator for ReadIterator<'a, Ctx, Type>
{
    fn len(&self) -> usize {
        self.bytes.map(|bytes| bytes.len()).unwrap_or_default()
    }
}
