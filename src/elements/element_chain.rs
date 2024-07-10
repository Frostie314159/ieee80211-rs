//! This module contains the element chain.
//!
//! An element chain is a data structure, which can contain distinct types, while only requiring the minimal amount of data necessary to store the elements, unlike an array of enums.
//! ## Example
//! ```
//! use ieee80211::elements::{element_chain::ElementChainEnd, SSIDElement};
//!
//! let chain = ElementChainEnd::new(SSIDElement::new_unchecked("OpenRF"));
//! ```
//! ## How it works
//! When you create an element chain, you start at [ElementChainEnd]. Like [ElementChainLink] it implements [ChainElement].
//! When you call [ChainElement::append], your original type is consumed and what you get back is [ChainElement::Appended].
//! Internally, an [ElementChainLink], creates a new version of itself, by moving it's inner value and calling [ChainElement::append] on it's child.
//! This repeats until an [ElementChainEnd] is reached. This is technically not recursion, since every element of the chain is a distinct type.
//! ## Disclaimer
//! There are other crates implementing this concept, like [object-chain](https://crates.io/crates/object-chain) and [typechain](https://crates.io/crates/typechain), however both didn't fit the needs of this project.

use core::marker::PhantomData;

use scroll::{
    ctx::{MeasureWith, TryIntoCtx},
    Endian, Pwrite,
};

use super::{
    Element, ElementID, RawIEEE80211Element, TypedIEEE80211Element, TypedIEEE80211ExtElement,
    VendorSpecificElement,
};

/// This trait represents a singular element of the chain.
pub trait ChainElement {
    /// The type produced, by appending an element to this element.
    type Appended<Appendee>: ChainElement;

    /// Append a new element to the chain.
    fn append<T>(self, value: T) -> Self::Appended<T>;
}

/// This is the end of a chain.
///
/// Counterintuitively it's the point where you create a new chain.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub struct ElementChainEnd<Inner> {
    /// The last element of the chain.
    pub inner: Inner,
}
impl<Inner: Element> ElementChainEnd<Inner> {
    #[inline]
    pub const fn new(inner: Inner) -> Self {
        Self { inner }
    }
}

impl<Inner> ChainElement for ElementChainEnd<Inner> {
    type Appended<Appendee> = ElementChainLink<Inner, ElementChainEnd<Appendee>>;
    #[inline]
    fn append<T>(self, value: T) -> Self::Appended<T> {
        ElementChainLink {
            inner: self.inner,
            next: ElementChainEnd { inner: value },
        }
    }
}

impl<Inner> MeasureWith<()> for ElementChainEnd<Inner>
where
    Inner: Element,
{
    #[inline]
    fn measure_with(&self, ctx: &()) -> usize {
        self.inner.measure_with(ctx) + if Inner::ELEMENT_ID.is_ext() { 3 } else { 2 }
    }
}
impl MeasureWith<()> for ElementChainEnd<RawIEEE80211Element<'_>> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.inner.slice.len() + 2
    }
}

impl<Inner> TryIntoCtx for ElementChainEnd<Inner>
where
    Inner: Element,
{
    type Error = scroll::Error;
    #[inline]
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        // TODO: Move this to shared code somehow.
        match Inner::ELEMENT_ID {
            ElementID::Id(id) => buf.pwrite(
                TypedIEEE80211Element {
                    tlv_type: id,
                    payload: self.inner,
                    _phantom: PhantomData,
                },
                0,
            ),
            ElementID::ExtId(ext_id) => buf.pwrite(
                TypedIEEE80211Element {
                    tlv_type: 0xff,
                    payload: TypedIEEE80211ExtElement {
                        ext_id,
                        payload: self.inner,
                    },
                    _phantom: PhantomData,
                },
                0,
            ),
            ElementID::VendorSpecific { oui, subtype } => buf.pwrite(
                TypedIEEE80211Element {
                    tlv_type: 0xdd,
                    payload: VendorSpecificElement {
                        oui,
                        subtype,
                        payload: self.inner,
                        _phantom: PhantomData,
                    },
                    _phantom: PhantomData,
                },
                0,
            ),
        }
    }
}
impl TryIntoCtx for ElementChainEnd<RawIEEE80211Element<'_>> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        buf.pwrite_with(self.inner, 0, Endian::Little)
    }
}

/// A link in the element chain.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub struct ElementChainLink<Inner, Child: ChainElement> {
    /// The current element.
    pub inner: Inner,
    /// The next element.
    pub next: Child,
}
impl<Inner, Child: ChainElement> ChainElement for ElementChainLink<Inner, Child> {
    type Appended<Appendee> = ElementChainLink<Inner, <Child as ChainElement>::Appended<Appendee>>;
    #[inline]
    fn append<T>(self, value: T) -> Self::Appended<T> {
        ElementChainLink {
            inner: self.inner,
            next: self.next.append(value),
        }
    }
}

impl<Inner, Child> MeasureWith<()> for ElementChainLink<Inner, Child>
where
    Inner: Element,
    Child: TryIntoCtx<Error = scroll::Error> + MeasureWith<()> + ChainElement,
{
    #[inline]
    fn measure_with(&self, ctx: &()) -> usize {
        self.inner.measure_with(ctx)
            + if Inner::ELEMENT_ID.is_ext() { 3 } else { 2 }
            + self.next.measure_with(ctx)
    }
}
impl<Child> MeasureWith<()> for ElementChainLink<RawIEEE80211Element<'_>, Child>
where
    Child: TryIntoCtx<Error = scroll::Error> + MeasureWith<()> + ChainElement,
{
    fn measure_with(&self, ctx: &()) -> usize {
        self.inner.slice.len() + 2 + self.next.measure_with(ctx)
    }
}

impl<Inner, Child> TryIntoCtx for ElementChainLink<Inner, Child>
where
    Inner: Element,
    Child: TryIntoCtx<Error = scroll::Error> + ChainElement,
{
    type Error = scroll::Error;
    #[inline]
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;
        match Inner::ELEMENT_ID {
            ElementID::Id(id) => buf.gwrite(
                TypedIEEE80211Element {
                    tlv_type: id,
                    payload: self.inner,
                    _phantom: PhantomData,
                },
                &mut offset,
            )?,
            ElementID::ExtId(ext_id) => buf.gwrite(
                TypedIEEE80211Element {
                    tlv_type: 0xff,
                    payload: TypedIEEE80211ExtElement {
                        ext_id,
                        payload: self.inner,
                    },
                    _phantom: PhantomData,
                },
                &mut offset,
            )?,
            ElementID::VendorSpecific { oui, subtype } => buf.gwrite(
                TypedIEEE80211Element {
                    tlv_type: 0xdd,
                    payload: VendorSpecificElement {
                        oui,
                        subtype,
                        payload: self.inner,
                        _phantom: PhantomData,
                    },
                    _phantom: PhantomData,
                },
                &mut offset,
            )?,
        };
        buf.gwrite(self.next, &mut offset)?;

        Ok(offset)
    }
}
impl<Child> TryIntoCtx for ElementChainLink<RawIEEE80211Element<'_>, Child>
where
    Child: TryIntoCtx<Error = scroll::Error> + ChainElement,
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;
        buf.gwrite_with(self.inner, &mut offset, Endian::Little)?;
        buf.gwrite(self.next, &mut offset)?;

        Ok(offset)
    }
}
