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

use scroll::{
    ctx::{MeasureWith, TryIntoCtx},
    Endian, Pwrite,
};

use super::{Element, RawIEEE80211Element, WrappedIEEE80211Element};

/// This trait represents a singular element of the chain.
pub trait ChainElement {
    /// The type produced, by appending an element to this element.
    type Appended<Appendee>: ChainElement;

    /// Append a new element to the chain.
    fn append<T>(self, value: T) -> Self::Appended<T>;
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
/// This is the end of a chain.
///
/// Counterintuitively it's the point where you create a new chain.
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
        Inner::ELEMENT_ID.element_header_length() + self.inner.measure_with(ctx)
    }
}
impl MeasureWith<()> for ElementChainEnd<RawIEEE80211Element<'_>> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.inner.slice.len()
    }
}

impl<Inner> TryIntoCtx for ElementChainEnd<Inner>
where
    Inner: Element,
{
    type Error = scroll::Error;
    #[inline]
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        buf.pwrite(WrappedIEEE80211Element(self.inner), 0)
    }
}
impl TryIntoCtx for ElementChainEnd<RawIEEE80211Element<'_>> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        buf.pwrite_with(self.inner, 0, Endian::Little)
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
/// A link in the element chain.
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
        Inner::ELEMENT_ID.element_header_length()
            + self.inner.measure_with(ctx)
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
        buf.gwrite(WrappedIEEE80211Element(self.inner), &mut offset)?;
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
#[macro_export]
/// Generate an element chain from the provided elements.
///
/// ```
/// use ieee80211::{element_chain, ssid, elements::{DSSSParameterSetElement, element_chain::ChainElement}};
///
/// let _element_chain = element_chain! {
///     ssid!("Test"),
///     DSSSParameterSetElement {
///         current_channel: 1
///     }
/// };
/// ```
macro_rules! element_chain {
    () => {
        ::ieee80211::common::Empty
    };
    ($element:expr) => {
        {
            use ieee80211::elements::element_chain::{ChainElement, ElementChainEnd};
            ElementChainEnd {
                inner: $element
            }
        }
    };
    (
        $current_element:expr
        $(,$element:expr)+
    ) => {
        {
            use ieee80211::elements::element_chain::{ChainElement, ElementChainLink};
            ElementChainLink {
                inner: $current_element,
                next: ::ieee80211::element_chain! ($($element),*)
            }
        }
    };
}
