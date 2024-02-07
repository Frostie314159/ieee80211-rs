use core::marker::PhantomData;

use scroll::Endian;

use crate::common::read_iterator::ReadIterator;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RateIterator<'a, RateType: From<u8>> {
    pub(crate) bytes: ReadIterator<'a, Endian, u8>,
    _phantom: PhantomData<RateType>,
}
impl<'a, RateType: From<u8>> RateIterator<'a, RateType> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes: ReadIterator::new(bytes),
            _phantom: PhantomData,
        }
    }
}
impl<'a, RateType: From<u8>> Iterator for RateIterator<'a, RateType> {
    type Item = RateType;
    fn next(&mut self) -> Option<Self::Item> {
        self.bytes.next().map(Into::into)
    }
}
