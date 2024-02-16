use core::{fmt::Debug, marker::PhantomData};

use macro_bits::serializable_enum;
use scroll::{Endian, Pwrite};
use tlv_rs::{raw_tlv::RawTLV, TLV};

use crate::common::read_iterator::ReadIterator;

use self::{
    ht_cap_oper::{HTCapabilitiesElement, HTOperationElement},
    rates::{EncodedRate, ExtendedSupportedRatesElement, RatesReadIterator, SupportedRatesElement},
};
/// This module contains the elements, which are found in the body of some frames.
/// If an element only consists of one struct, like the [ssid::SSIDTLV], they are re-exported, otherwise they get their own module.
mod dsss_parameter_set;
pub use dsss_parameter_set::DSSSParameterElement;
pub mod rates;
mod ssid;
pub use ssid::SSIDElement;
mod bss_load;
pub mod ht_cap_oper;
pub use bss_load::*;
mod ibss_parameter_set;
pub use ibss_parameter_set::IBSSParameterSetElement;

/// A raw TLV.
pub type RawIEEE80211Element<'a> = RawTLV<'a, u8, u8>;
type TypedIEEE80211Element<Payload> = TLV<u8, u8, u8, Payload>;

macro_rules! elements {
    (
        $(
            #[$meta_var:meta]
        )*
        pub enum $enum_name:ident <$lt:lifetime $(, $generic:ident: $($trait_bound:path),* = $default:ty)*> {
            $(
                $(
                    #[$sub_meta_var:meta]
                )*
                $element_type_name:ident : $element_type_value:expr => $element_type:ty
            ),*
        }
    ) => {
        serializable_enum! {
            #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
            /// The type of an IEEE 802.11 Information Element.
            pub enum IEEE80211ElementID: u8{
                $(
                    $element_type_name => $element_type_value,
                )*
                #[default]
                Reserved => 0xe3
            }
        }
        $(
            #[$meta_var]
        )*
        pub enum $enum_name <$lt, $($generic = $default),*> 
        where
            $(
                $generic: $($trait_bound + )*
            ),*
        {
            $(
                $(
                    #[$sub_meta_var]
                )*
                $element_type_name($element_type),
            )*
            Unknown(RawIEEE80211Element<$lt>)
        }
        impl<$lt $(, $generic: $($trait_bound + )*)*> $enum_name<$lt $(, $generic)*> {
            pub const fn get_element_type(&self) -> IEEE80211ElementID {
                match self {
                    $(
                        Self::$element_type_name(_) => IEEE80211ElementID::$element_type_name,
                    )*
                    Self::Unknown(raw_tlv) => IEEE80211ElementID::Unknown(raw_tlv.tlv_type)
                }
            }
        }
        impl<$lt $(, $generic: $($trait_bound + )*)*> ::scroll::ctx::MeasureWith<()> for $enum_name<$lt $(, $generic)*> {
            fn measure_with(&self, ctx: &()) -> usize {
                2 + match self {
                    $(
                        Self::$element_type_name(tlv) => tlv.measure_with(ctx),
                    )*
                    Self::Unknown(raw_tlv) => raw_tlv.slice.len()
                }
            }
        }
        impl<$lt> ::scroll::ctx::TryFromCtx<$lt> for $enum_name<$lt> {
            type Error = ::scroll::Error;
            fn try_from_ctx(from: &$lt [u8], _ctx: ()) -> Result<(Self, usize), ::scroll::Error> {
                let (tlv, len) =
                    <RawIEEE80211Element<'a> as ::scroll::ctx::TryFromCtx<'a, Endian>>::try_from_ctx(from, Endian::Little)?;
                Ok((
                    match tlv.tlv_type {
                        $(
                            $element_type_value => Self::$element_type_name(::scroll::ctx::TryFromCtx::try_from_ctx(tlv.slice, ()).map(|(tlv, _)| tlv)?),
                        )*
                        _ => Self::Unknown(tlv)
                    },
                    len
                ))
            }
        }
        impl<$lt $(, $generic: $($trait_bound + )*)*> ::scroll::ctx::TryIntoCtx for $enum_name<$lt $(, $generic)*> {
            type Error = ::scroll::Error;
            fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, ::scroll::Error> {
                match self {
                    $(
                        Self::$element_type_name(payload) => buf.pwrite(TypedIEEE80211Element {
                            tlv_type: $element_type_value,
                            payload,
                            _phantom: PhantomData
                        }, 0),
                    )*
                    Self::Unknown(payload) => buf.pwrite(payload, 0)
                }

            }
        }
        pub trait ToElement<$lt $(, $generic: $($trait_bound + )* = $default)*> {
            fn to_element(self) -> $enum_name<$lt $(, $generic)*>;
        }
        macro_rules! to_element_impl {
            ($element_type_2:ty, $element_type_name_2:ident) => {
                impl<$lt $(, $generic: $($trait_bound + )*)*> ToElement<$lt $(, $generic)*> for $element_type_2 {
                    fn to_element(self) -> $enum_name<$lt $(, $generic)*> {
                        $enum_name::$element_type_name_2(self)
                    }
                }
            };
        }
        $(
            to_element_impl!($element_type, $element_type_name);
        )*
    };
}

elements! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// This enum contains all possible elements.
    /// For documentation on the individual elements please refer to the docs provided for their structs.
    /// They are ordered by their ID.
    pub enum IEEE80211Element<
        'a,
        RateIterator: IntoIterator<Item = EncodedRate>, Clone = RatesReadIterator<'a>,
        ExtendedRateIterator: IntoIterator<Item = EncodedRate>, Clone = RatesReadIterator<'a>
    > {
        SSID: 0x00 => SSIDElement<'a>,
        SupportedRates: 0x01 => SupportedRatesElement<RateIterator>,
        DSSSParameterSet: 0x03 => DSSSParameterElement,
        IBSSParameterSet: 0x06 => IBSSParameterSetElement,
        BSSLoad: 0x0b => BSSLoadElement,
        HTCapabilities: 0x2d => HTCapabilitiesElement,
        ExtendedSupportedRates: 0x32 => ExtendedSupportedRatesElement<ExtendedRateIterator>,
        HTOperation: 0x3d => HTOperationElement
    }
}
/// This is an iterator over the elements contained in the body of a frame.
///
/// It's short circuiting.
pub type ElementReadIterator<'a> = ReadIterator<'a, (), IEEE80211Element<'a>>;
