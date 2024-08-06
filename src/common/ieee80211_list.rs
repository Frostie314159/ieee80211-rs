use core::{
    fmt::Debug,
    iter::{repeat, Repeat, Scan},
    marker::PhantomData,
    mem::size_of,
};

use num::{cast::AsPrimitive, Integer};
use scroll::{
    ctx::{MeasureWith, SizeWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

/// A generic trait over a list of items, as it is represented in IEEE 802.11.
///
/// It was originally developed for the [RSN element](crate::elements::rsn::RSNElement), but maybe reused in the future.
pub trait IEEE80211List<InnerItem, ElementCountType>
where
    Self: Clone,
{
    type Iter: Iterator<Item = InnerItem>;

    /// Returns the amount of elements in the list.
    fn element_count(&self) -> ElementCountType;
    /// Returns an [Iterator] over the elements of the list.
    ///
    /// The reason this isn't done through the [IntoIterator] trait, is because it allows us to specialize the implementations.
    fn iter(self) -> Self::Iter;

    /// Returns the size in bytes of the list when written out.
    fn size_in_bytes(&self) -> usize;

    /// Compare two lists.
    fn eq<Rhs: IEEE80211List<InnerItem, ElementCountType> + Clone>(&self, other: &Rhs) -> bool
    where
        InnerItem: PartialEq,
    {
        self.clone().iter().eq(other.clone().iter())
    }
}
impl<T, InnerItem, ElementCountType> IEEE80211List<InnerItem, ElementCountType> for T
where
    T: IntoIterator<Item = InnerItem> + Clone,
    <T as IntoIterator>::IntoIter: ExactSizeIterator,
    ElementCountType: Integer + Copy + AsPrimitive<usize> + 'static,
    usize: AsPrimitive<ElementCountType>,
    InnerItem: SizeWith,
{
    type Iter = T::IntoIter;
    fn element_count(&self) -> ElementCountType {
        self.clone().into_iter().len().as_()
    }
    fn iter(self) -> Self::Iter {
        self.into_iter()
    }
    fn size_in_bytes(&self) -> usize {
        size_of::<ElementCountType>() + InnerItem::size_with(&()) * self.element_count().as_()
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
/// An IEEE 802.11 list returned by reading.
pub struct IEEE80211ReadList<'a, InnerItem, ElementCountType, const ITEM_SIZE: usize> {
    pub(crate) bytes: &'a [u8],
    _phantom: PhantomData<(InnerItem, ElementCountType)>,
}
impl<'a, InnerItem, ElementCountType, const ITEM_SIZE: usize> Debug
    for IEEE80211ReadList<'a, InnerItem, ElementCountType, ITEM_SIZE>
where
    InnerItem: SizeWith + TryFromCtx<'a, Error = scroll::Error> + Clone + Debug,
    ElementCountType:
        TryFromCtx<'a, Endian, Error = scroll::Error> + Integer + AsPrimitive<usize> + Copy,
    usize: AsPrimitive<ElementCountType>,
    Self: IEEE80211List<InnerItem, ElementCountType>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.clone().iter()).finish()
    }
}
impl<'a, InnerItem, ElementCountType, const ITEM_SIZE: usize>
    IEEE80211List<InnerItem, ElementCountType>
    for IEEE80211ReadList<'a, InnerItem, ElementCountType, ITEM_SIZE>
where
    InnerItem: SizeWith + TryFromCtx<'a, Error = scroll::Error> + Clone,
    ElementCountType:
        TryFromCtx<'a, Endian, Error = scroll::Error> + Integer + AsPrimitive<usize> + Copy,
    usize: AsPrimitive<ElementCountType>,
{
    type Iter = Scan<Repeat<()>, &'a [u8], fn(&mut &'a [u8], ()) -> Option<InnerItem>>;
    fn element_count(&self) -> ElementCountType {
        (self.bytes.len() / ITEM_SIZE).as_()
    }
    fn iter(self) -> Self::Iter {
        // This sidesteps the issue, that the `ReadIterator` requires an implementation of `MeasureWith<()>`, which isn't present for numeric types (yet).
        // Numeric types are requried for the PMKID list of the `RSNElement`, for which most of this code was written.
        repeat(()).scan(self.bytes, |bytes, _| {
            let mut offset = 0;
            match bytes.gread(&mut offset) {
                Ok(out) => {
                    *bytes = &bytes[offset..];
                    Some(out)
                }
                Err(_) => None,
            }
        })
    }
    fn size_in_bytes(&self) -> usize {
        size_of::<ElementCountType>() + self.bytes.len()
    }
}
impl<
        'a,
        InnerItem: 'a,
        ElementCountType: TryFromCtx<'a, Endian, Error = scroll::Error> + Into<usize>,
        const ITEM_SIZE: usize,
    > TryFromCtx<'a> for IEEE80211ReadList<'a, InnerItem, ElementCountType, ITEM_SIZE>
{
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;
        let count: usize = from
            .gread_with::<ElementCountType>(&mut offset, Endian::Little)?
            .into();
        let bytes = from.gread_with(&mut offset, count * ITEM_SIZE)?;
        Ok((
            Self {
                bytes,
                _phantom: PhantomData,
            },
            offset,
        ))
    }
}
impl<'a, InnerItem, ElementCountType, const ITEM_SIZE: usize> TryIntoCtx
    for IEEE80211ReadList<'a, InnerItem, ElementCountType, ITEM_SIZE>
where
    InnerItem: 'a,
    ElementCountType: TryIntoCtx<Endian, Error = scroll::Error> + Copy + 'static,
    usize: AsPrimitive<ElementCountType>,
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(
            <usize as AsPrimitive<ElementCountType>>::as_(self.bytes.len() / ITEM_SIZE),
            &mut offset,
            Endian::Little,
        )?;
        buf.gwrite(self.bytes, &mut offset)?;

        Ok(offset)
    }
}
impl<'a, InnerItem, ElementCountType, const ITEM_SIZE: usize> MeasureWith<()>
    for IEEE80211ReadList<'a, InnerItem, ElementCountType, ITEM_SIZE>
{
    fn measure_with(&self, _ctx: &()) -> usize {
        self.bytes.len()
    }
}
