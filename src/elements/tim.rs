//! This module provides support for the Traffic Indication Map (TIM).
//!
//! ## What is a TIM and a DTIM
//! The TIM was introduced as a power saving mechanism, to allow STAs to enter a low power mode, in which they only occasionally switch on their receiver to listen for beacons.
//! When a STA is in this low power mode, it looks at the TIM inside the received beacon, to see if traffic is buffered for itself.
//! A Delivery Traffic Indication Map (DTIM) is the same as a normal TIM, except that it is transmitted only in some beacon frames.
//! The [TIMElement::dtim_period] field specifies how many beacon intervals pass between successive DTIM's, and [TIMElement::dtim_count] is the amount of beacon intervals until the next DTIM.
//! At every DTIM, buffered multicast MPDU's will be transmitted.
//!
//! ## Example
//! This example demonstrates the different ways to construct a [TIMElement].
//! Only one way of using [tim_bitmap](crate::tim_bitmap) is demonstrated here, but there are more examples in it's own documentation.
//! The use of [TIMBitmap::new_raw]
//! ```
//! use ieee80211::{elements::tim::{TIMElement, TIMBitmap}, tim_bitmap, aid};
//!
//! // The bitmap for this TIMElement is generated at compile time.
//! let _static_tim_element = TIMElement {
//!     dtim_count: 2, // Two beacon intervals until the next DTIM.
//!     dtim_period: 3, // Every third beacon is a DTIM.
//!     bitmap: Some(tim_bitmap! [
//!         42,
//!         69,
//!         1337
//!     ]),
//!     ..Default::default()
//! };
//! // The bitmap for this TIMElement is generated at run time.
//! let _run_time_tim_element = TIMElement {
//!     dtim_count: 2, // Two beacon intervals until the next DTIM.
//!     dtim_period: 3, // Every third beacon is a DTIM.
//!     bitmap: Some(TIMBitmap::new_static(
//!         false, // No broadcast traffic is buffered.
//!         [
//!             aid!(42), // We use the aid! macro here, since we know these numbers at compile time.
//!             aid!(69),
//!             aid!(1337)
//!         ]
//!     )),
//!     ..Default::default()
//! };
//! ```

use core::{
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::Deref,
};

use bitfield_struct::bitfield;
use macro_bits::{bit, check_bit, set_bit};
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::common::AssociationID;

use super::{Element, ElementID};

#[bitfield(u8)]
#[derive(PartialEq, Eq, Hash)]
/// The bitmap control field of a [TIMElement].
pub struct TIMBitmapControl {
    /// Indicates, if multicast traffic is buffered at the AP.
    pub traffic_indicator: bool,
    #[bits(7)]
    /// This is the offset of the bitmap, which is equal to N1 divided by two.
    ///
    /// In most cases, setting it through [Self::n1], [Self::set_n1] or [Self::with_n1] is better, since it performs the conversion automatically.
    pub bitmap_offset: u8,
}
impl TIMBitmapControl {
    #[inline]
    /// Get N1 from the bitmap offset.
    ///
    /// This is just multiplies the bitmap offset with two.
    pub const fn n1(&self) -> u8 {
        self.bitmap_offset() * 2
    }
    #[inline]
    /// Set the bitmap offset, with N1.
    pub fn set_n1(&mut self, n1: u8) {
        self.set_bitmap_offset(n1 / 2)
    }
    #[inline]
    /// Set the bitmap offset, with N1.
    pub const fn with_n1(self, n1: u8) -> Self {
        self.with_bitmap_offset(n1 / 2)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
/// A static bitmap for the [TIMBitmap].
///
/// This exists to hold the partial virtual bitmap and N2, since we somehow needed to have an array holding the partial virtual bitmap, but don't want to write all of it.
pub struct StaticBitmap(pub [u8; 251], pub usize);
impl StaticBitmap {
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..=self.1]
    }
}
impl Debug for StaticBitmap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.deref()))
    }
}
impl Deref for StaticBitmap {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}
impl Default for StaticBitmap {
    fn default() -> Self {
        Self([0; 251], 0)
    }
}
impl MeasureWith<()> for StaticBitmap {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.1
    }
}
impl TryIntoCtx for StaticBitmap {
    type Error = scroll::Error;
    #[inline]
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        buf.pwrite(self.as_bytes(), 0)
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
/// This is required to make the tim_bitmap! macro work in const, since range based indexing isn't stable yet.
pub struct ConstBitmap(pub &'static [u8; 251], pub usize, pub usize);
impl ConstBitmap {
    pub fn as_bytes(&self) -> &'static [u8] {
        &self.0[self.1..=self.2]
    }
}
impl Deref for ConstBitmap {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}
impl Debug for ConstBitmap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.deref()))
    }
}
impl Default for ConstBitmap {
    fn default() -> Self {
        Self(&[0; 251], 0, 0)
    }
}
impl MeasureWith<()> for ConstBitmap {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.as_bytes().len()
    }
}
impl TryIntoCtx for ConstBitmap {
    type Error = scroll::Error;
    #[inline]
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        buf.pwrite(self.as_bytes(), 0)
    }
}

#[derive(Clone, Copy, Debug, Default, Hash)]
/// The complete bitmap of a [TIMElement], including the [TIMBitmapControl].
///
/// It is **strongly** advised, that you don't set the fields in this struct directly, but much rather use [Self::new_static] or [tim_bitmap](crate::tim_bitmap).
/// For reading the bitmap you should use [Self::aid_iter].
pub struct TIMBitmap<Bitmap> {
    bitmap_control: TIMBitmapControl,
    partial_virtual_bitmap: Option<Bitmap>,
}
impl<Bitmap> TIMBitmap<Bitmap> {
    #[inline]
    /// Returns the traffic indicator bit.
    ///
    /// This is
    pub const fn traffic_indicator(&self) -> bool {
        self.bitmap_control.traffic_indicator()
    }
    #[inline]
    #[doc(hidden)]
    // Creates a TIMBitmap, without performing any checks.
    // If the passed data is invalid, this may cause aid_iter to panic.
    pub const fn new_unchecked(
        bitmap_control: TIMBitmapControl,
        partial_virtual_bitmap: Option<Bitmap>,
    ) -> Self {
        Self {
            bitmap_control,
            partial_virtual_bitmap,
        }
    }

    #[inline]
    /// Creates a new [TIMBitmap] from raw values.
    ///
    /// # Returns
    /// This returns [None], if the length of the bitmap plus `N1` is larger than 251, as that could cause [Self::aid_iter] to panic, and [Some] in all other cases.
    pub fn new_raw(
        bitmap_control: TIMBitmapControl,
        partial_virtual_bitmap: Option<Bitmap>,
    ) -> Option<Self>
    where
        Bitmap: Deref<Target = [u8]>,
    {
        if let Some(ref partial_virtual_bitmap) = partial_virtual_bitmap {
            if partial_virtual_bitmap.len() + bitmap_control.n1() as usize > 251 {
                return None;
            }
        }
        Some(Self {
            bitmap_control,
            partial_virtual_bitmap,
        })
    }
}
impl TIMBitmap<StaticBitmap> {
    /// Create a static [TIMBitmap].
    ///
    /// This returns a [StaticBitmap], which holds a fixed size array and the length.
    ///
    /// # Note
    /// If more than 2007 [AssociationID]'s are present, any remaining will be truncated.
    pub fn new_static(
        multicast_traffic_buffered: bool,
        association_ids: impl IntoIterator<Item = AssociationID>,
    ) -> TIMBitmap<StaticBitmap> {
        let mut partial_virtual_bitmap = [0u8; 251];
        // We set N1 and N2 to opposing values.
        let mut n1 = 251;
        let mut n2 = 0;

        for aid in association_ids
            .into_iter()
            .take(AssociationID::MAX_AID as usize)
        {
            let aid = aid.aid();

            let byte_index = aid as usize / 8;
            let bit_index = aid % 8;
            set_bit!(partial_virtual_bitmap[byte_index], bit!(bit_index));
            if byte_index < n1 && aid != 0 {
                n1 = byte_index;
            }
            if byte_index > n2 {
                n2 = byte_index;
            }
        }
        if n1 == 0 && multicast_traffic_buffered {
            set_bit!(partial_virtual_bitmap[0], bit!(1));
        }

        TIMBitmap {
            bitmap_control: TIMBitmapControl::new()
                .with_traffic_indicator(multicast_traffic_buffered)
                .with_n1(n1 as u8),
            partial_virtual_bitmap: Some(StaticBitmap(partial_virtual_bitmap, n2)),
        }
    }
}
impl<Bitmap: Deref<Target = [u8]>> TIMBitmap<Bitmap> {
    /// Returns an iterator over the [AssociationID]'s, for which traffic is buffered.
    ///
    /// # Note
    /// AID zero isn't included, since it isn't a valid [AssociationID].
    pub fn aid_iter(&self) -> Option<impl Iterator<Item = AssociationID> + '_> {
        self.partial_virtual_bitmap.as_deref().map(|bytes| {
            (1..(bytes.len() * 8)).filter_map(|aid| {
                if check_bit!(bytes[aid / 8], bit!(aid % 8)) {
                    AssociationID::new_checked(self.bitmap_control.n1() as u16 * 8 + aid as u16)
                } else {
                    None
                }
            })
        })
    }
}
impl<Bitmap: Deref<Target = [u8]>> Display for TIMBitmap<Bitmap> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut debug_list = f.debug_list();

        if self.bitmap_control.traffic_indicator() {
            debug_list.entry(&0u16);
        }
        if let Some(aid_iter) = self.aid_iter() {
            debug_list.entries(aid_iter)
        } else {
            &mut debug_list
        }
        .finish()
    }
}
#[cfg(feature = "defmt")]
impl<Bitmap: Deref<Target = [u8]>> defmt::Format for TIMBitmap<Bitmap> {
    fn format(&self, fmt: defmt::Formatter) {
        if let Some(mut aid_iter) = self.aid_iter() {
            if let Some(first) = aid_iter.next() {
                defmt::write!(fmt, "[{}", first.aid());
                for aid in aid_iter {
                    defmt::write!(fmt, ", {}", aid.aid());
                }
                defmt::write!(fmt, "]")
            }
        }
        defmt::write!(fmt, "[]")
    }
}
impl<LhsBitmap: Deref<Target = [u8]>, RhsBitmap: Deref<Target = [u8]>>
    PartialEq<TIMBitmap<RhsBitmap>> for TIMBitmap<LhsBitmap>
{
    fn eq(&self, other: &TIMBitmap<RhsBitmap>) -> bool {
        self.bitmap_control == other.bitmap_control
            && self.partial_virtual_bitmap.as_deref() == other.partial_virtual_bitmap.as_deref()
    }
}
impl<Bitmap: Deref<Target = [u8]>> Eq for TIMBitmap<Bitmap> {}
impl<'a> TryFromCtx<'a> for TIMBitmap<&'a [u8]> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let bitmap_control =
            TIMBitmapControl::from_bits(from.gread_with(&mut offset, Endian::Little)?);
        let partial_virtual_bitmap = Some(&from[offset..]);

        Ok((
            Self {
                bitmap_control,
                partial_virtual_bitmap,
            },
            from.len(),
        ))
    }
}
impl<Bitmap: MeasureWith<()>> MeasureWith<()> for TIMBitmap<Bitmap> {
    fn measure_with(&self, ctx: &()) -> usize {
        1 + if let Some(ref partial_virtual_bitmap) = self.partial_virtual_bitmap {
            partial_virtual_bitmap.measure_with(ctx)
        } else {
            0
        }
    }
}
impl<Bitmap: TryIntoCtx<Error = scroll::Error>> TryIntoCtx for TIMBitmap<Bitmap> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(self.bitmap_control.into_bits(), &mut offset, Endian::Little)?;
        if let Some(partial_virtual_bitmap) = self.partial_virtual_bitmap {
            buf.gwrite(partial_virtual_bitmap, &mut offset)?;
        }

        Ok(offset)
    }
}

#[macro_export]
/// Generate a static [TIMBitmap].
///
/// If you're using a fixed TIM (for whatever reason), this can be used to produce a [TIMBitmap], which internally just holds a static byte slice.
/// It will also enforce bounds checks, but not check if an AID appears twice.
///
/// ## Note
/// This does not currently work in const contexts due to range based indexing not being const.
/// ## Usage
/// There are three ways, this macro can be used.
///
/// ### Multicast Only
/// This will set the traffic indicator to `true` and the partial virtual to [None].
/// ```
/// use ieee80211::tim_bitmap;
///
/// let bitmap = tim_bitmap!(0);
/// assert!(bitmap.traffic_indicator());
/// ```
///
/// ### Multiple AIDs
/// This allows setting multiple AIDs.
/// ```
/// use ieee80211::tim_bitmap;
///
/// let bitmap = tim_bitmap! [
///     42,
///     1337
/// ];
/// assert!(!bitmap.traffic_indicator());
/// ```
/// ### AID Range
/// This allows specifying a range of AIDs, and is faster to evaluate.
/// ```
/// use ieee80211::tim_bitmap;
///
/// let _bitmap = tim_bitmap!(0 => 1337);
/// ```
macro_rules! tim_bitmap {
    (0) => {
        {
            use ::ieee80211::elements::tim::{TIMBitmapControl, TIMBitmap};

            TIMBitmap::<&[u8]>::new_unchecked(TIMBitmapControl::new().with_traffic_indicator(true), None)
        }
    };
    ($($aid:expr),*) => {
        {
            use ::ieee80211::{macro_bits::{set_bit, bit}, elements::tim::{TIMBitmapControl, TIMBitmap, ConstBitmap}, common::AssociationID};
            const TRAFFIC_INDICATOR: bool = {
                let mut traffic_indicator = false;
                $(
                    traffic_indicator |= $aid == 0;
                )*
                traffic_indicator
            };
            const BITMAP: ([u8; 251], usize, usize) = {
                let mut partial_virtual_bitmap = [0u8; 251];
                // The lowest byte.
                let mut n1 = 251;
                // The highest byte.
                let mut n2 = 0;

                $(
                    assert!($aid <= AssociationID::MAX_AID, "An AID higher than 2007 is invalid.");
                    // We actually set bit zero, if that AID is present, but it doesn't count towards N1.
                    let byte_index = ($aid / 8) as usize;
                    let bit_index = ($aid % 8) as usize;
                    set_bit!(partial_virtual_bitmap[byte_index], bit!(bit_index));
                    // We're looking for the lowest AID, that isn't zero, since that's encoded in the traffic indicator already.
                    if byte_index < n1 && $aid != 0 {
                        n1 = byte_index;
                    }
                    if byte_index > n2 {
                        n2 = byte_index;
                    }
                )*

                // Clearing the LSB rounds N1 down to the next even number, as required in the standard.
                (partial_virtual_bitmap, n1 & 0b1111_1110, n2)
            };
            const PARTIAL_VIRTUAL_BITMAP: [u8; 251] = BITMAP.0;
            // Const-eval would take even longer, if we unrolled the AID's twice more.
            const N1: usize = BITMAP.1;
            const N2: usize = BITMAP.2;

            TIMBitmap::new_unchecked(
                TIMBitmapControl::new().with_traffic_indicator(TRAFFIC_INDICATOR).with_n1(N1 as u8),
                Some(ConstBitmap(&PARTIAL_VIRTUAL_BITMAP, N1, N2))
            )
        }
    };
    ($min_aid:expr => $max_aid:expr) => {
        {
            use ::ieee80211::{macro_bits::{set_bit, bit}, elements::tim::{TIMBitmapControl, TIMBitmap, ConstBitmap}, common::AssociationID};
            const TRAFFIC_INDICATOR: bool = $min_aid == 0;
            const PARTIAL_VIRTUAL_BITMAP: [u8; 251] = {
                assert!($max_aid <= AssociationID::MAX_AID, "An AID higher than 2007 is invalid.");
                let mut partial_virtual_bitmap = [0u8; 251];

                let mut i = $min_aid;
                while i <= $max_aid {
                    set_bit!(partial_virtual_bitmap[i/8], bit!(i % 8));
                    i += 1;
                }

                partial_virtual_bitmap
            };
            const N1: usize = $min_aid / 8 & 0b1111_1110;
            const N2: usize = $max_aid / 8;

            TIMBitmap::new_unchecked(
                TIMBitmapControl::new().with_traffic_indicator(TRAFFIC_INDICATOR).with_n1(N1 as u8),
                Some(ConstBitmap(&PARTIAL_VIRTUAL_BITMAP, N1, N2))
            )
        }
    };
}

#[derive(Clone, Copy, Debug, Default, Hash)]
/// The Traffic Indication Map (TIM) element holds information, about the STAs, for which traffic is buffered at the AP.
///
/// If `dtim_count` field is larger than `dtim_period`, an error will be returned, when reading/writing.
pub struct TIMElement<'a, Bitmap = &'a [u8]> {
    /// The amount of beacon intervals until the next DTIM.
    pub dtim_count: u8,
    /// The number of beacon intervals between successive DTIM's.
    pub dtim_period: u8,
    /// The bitmap indicates the STAs, for which traffic is buffered.
    pub bitmap: Option<TIMBitmap<Bitmap>>,
    pub _phantom: PhantomData<&'a ()>,
}
impl<Bitmap> TIMElement<'_, Bitmap> {
    /// Check if the DTIM parameters are valid.
    const fn check_dtim_parameters(dtim_period: u8, dtim_count: u8) -> Result<(), scroll::Error> {
        if dtim_period < dtim_count {
            Err(scroll::Error::BadInput {
                size: 0,
                msg: "DTIM count is larger than DTIM period.",
            })
        } else {
            Ok(())
        }
    }
}
impl<Bitmap: Deref<Target = [u8]>> Display for TIMElement<'_, Bitmap> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut binding = f.debug_struct("TIMElement");
        let debug_struct = binding
            .field("dtim_count", &self.dtim_count)
            .field("dtim_period", &self.dtim_period);
        if let Some(ref bitmap) = self.bitmap {
            debug_struct.field("association_ids", &format_args!("{bitmap}"))
        } else {
            debug_struct
        }
        .finish()
    }
}
#[cfg(feature = "defmt")]
impl<Bitmap: Deref<Target = [u8]>> defmt::Format for TIMElement<'_, Bitmap> {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "TIMElement {{ dtim_count: {}, dtim_period: {}, bitmap: {} }}",
            self.dtim_count,
            self.dtim_period,
            self.bitmap
        )
    }
}
impl<LhsBitmap: Deref<Target = [u8]>, RhsBitmap: Deref<Target = [u8]>>
    PartialEq<TIMElement<'_, RhsBitmap>> for TIMElement<'_, LhsBitmap>
{
    fn eq(&self, other: &TIMElement<RhsBitmap>) -> bool {
        self.dtim_count == other.dtim_count
            && self.dtim_period == other.dtim_period
            && match (&self.bitmap, &other.bitmap) {
                (Some(lhs), Some(rhs)) => lhs == rhs,
                (None, None) => true,
                _ => false,
            }
    }
}
impl<Bitmap: Deref<Target = [u8]>> Eq for TIMElement<'_, Bitmap> {}
impl<'a> TryFromCtx<'a> for TIMElement<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let dtim_count = from.gread(&mut offset)?;
        let dtim_period = from.gread(&mut offset)?;
        Self::check_dtim_parameters(dtim_period, dtim_count)?;

        let bitmap = from.gread(&mut offset).ok();
        Ok((
            Self {
                dtim_count,
                dtim_period,
                bitmap,
                _phantom: PhantomData,
            },
            offset,
        ))
    }
}
impl<Bitmap: MeasureWith<()>> MeasureWith<()> for TIMElement<'_, Bitmap> {
    fn measure_with(&self, ctx: &()) -> usize {
        2 + if let Some(ref bitmap) = self.bitmap {
            bitmap.measure_with(ctx)
        } else {
            0
        }
    }
}
impl<Bitmap: TryIntoCtx<Error = scroll::Error>> TryIntoCtx for TIMElement<'_, Bitmap> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        Self::check_dtim_parameters(self.dtim_period, self.dtim_count)?;

        buf.gwrite(self.dtim_count, &mut offset)?;
        buf.gwrite(self.dtim_period, &mut offset)?;
        if let Some(bitmap) = self.bitmap {
            buf.gwrite(bitmap, &mut offset)?;
        }

        Ok(offset)
    }
}
impl<Bitmap> Element for TIMElement<'_, Bitmap>
where
    Self: MeasureWith<()> + TryIntoCtx<Error = scroll::Error>,
{
    const ELEMENT_ID: ElementID = ElementID::Id(5);
    type ReadType<'a> = TIMElement<'a>;
}
