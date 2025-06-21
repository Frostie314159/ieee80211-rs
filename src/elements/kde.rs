use core::marker::PhantomData;

use crate::elements::rsn::IEEE80211Pmkid;
use bitfield_struct::bitfield;
use mac_parser::MACAddress;
use scroll::{ctx::{MeasureWith, TryFromCtx, TryIntoCtx}, Endian, Pread, Pwrite};

use super::{Element, ElementID};

#[macro_export]
/// Define a key descriptor element.
macro_rules! define_kde {
    (
        $(#[$struct_meta:meta])*
        pub struct $kde_type:ident ($inner:ty) : $selector:expr, $size:expr $(, $optional_ctx:expr)?;
    ) => {
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $kde_type($inner);
        impl ::scroll::ctx::MeasureWith<()> for $kde_type {
            fn measure_with(&self, _ctx: &()) -> usize {
                $size
            }
        }
        impl ::scroll::ctx::TryFromCtx<'_> for $kde_type {
            type Error = ::scroll::Error;
            fn try_from_ctx(from: &[u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
                #[allow(unused)]
                let mut ctx = Default::default();
                $(ctx = $optional_ctx;)?
                <$inner>::try_from_ctx(from, ctx).map(|(inner, offset)| ($kde_type(inner), offset))
            }
        }
        impl ::scroll::ctx::TryIntoCtx<()> for $kde_type {
            type Error = ::scroll::Error;
            fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
                use ::scroll::Pwrite;
                #[allow(unused)]
                let mut ctx = Default::default();
                $(ctx = $optional_ctx;)?
                buf.pwrite_with(self.0, 0, ctx)
            }
        }
        impl $crate::elements::Element for $kde_type {
            const ELEMENT_ID: $crate::elements::ElementID = $crate::elements::ElementID::VendorSpecific {
                prefix: &[0x00, 0x0f, 0xac, $selector],
            };
            type ReadType<'a> = $kde_type;
        }
    };
}
define_kde! {
    pub struct MacAddressKde(MACAddress): 3, 6;
}
define_kde! {
    pub struct PmkidKde(IEEE80211Pmkid): 4, 16;
}
define_kde! {
    pub struct NonceKde([u8; 32]): 6, 32;
}
define_kde! {
    pub struct LifetimeKde(u32): 7, 4, Endian::Big;
}
#[bitfield(u16)]
pub struct GtkInfo {
    #[bits(2)]
    pub key_id: u8,
    pub tx: bool,
    #[bits(13)]
    pub __: u16,
}
pub struct GtkKde<'a, Gtk: AsRef<[u8]> = &'a [u8]> {
    pub gtk_info: GtkInfo,
    pub gtk: Gtk,
    pub _phantom: PhantomData<&'a ()>,
}
impl<'a> TryFromCtx<'a> for GtkKde<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;
        let gtk_info = GtkInfo::from_bits(from.gread_with(&mut offset, Endian::Little)?);
        let gtk = &from[offset..];
        offset = from.len();
        Ok((
            Self {
                gtk_info,
                gtk,
                _phantom: PhantomData,
            },
            offset,
        ))
    }
}
impl<Gtk: AsRef<[u8]>> MeasureWith<()> for GtkKde<'_, Gtk> {
    fn measure_with(&self, _ctx: &()) -> usize {
        2 + self.gtk.as_ref().len()
    }
}
impl<Gtk: AsRef<[u8]>> TryIntoCtx<()> for GtkKde<'_, Gtk> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(self.gtk_info.into_bits(), &mut offset, Endian::Little)?;
        buf.gwrite(self.gtk.as_ref(), &mut offset)?;

        Ok(offset)
    }
}
impl<Gtk: AsRef<[u8]>> Element for GtkKde<'_, Gtk> {
    const ELEMENT_ID: ElementID = ElementID::VendorSpecific { prefix: &[0x00, 0x0f, 0xac, 0x01] };
    type ReadType<'a> = GtkKde<'a>;
}
