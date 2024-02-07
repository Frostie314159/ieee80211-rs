use core::{fmt::Debug, marker::PhantomData};

use macro_bits::serializable_enum;
use scroll::{Endian, Pwrite};
use tlv_rs::{raw_tlv::RawTLV, TLV};

use crate::common::read_iterator::ReadIterator;

use self::rates::{
    EncodedExtendedRate, EncodedRate, ExtendedSupportedRatesTLV,
    ExtendedSupportedRatesTLVReadRateIterator, SupportedRatesTLV,
    SupportedRatesTLVReadRateIterator,
};
/// This module contains the elements, which are found in the body of some frames.
/// If an element only consists of one struct, like the [ssid::SSIDTLV], they are re-exported, otherwise they get their own module.
mod dsss_parameter_set;
pub use dsss_parameter_set::DSSSParameterSet;
pub mod rates;
mod ssid;
pub use ssid::SSIDTLV;
mod channel_switch_announcement;
pub mod ht_capabilities;

/// A raw TLV.
pub type RawIEEE80211TLV<'a> = RawTLV<'a, u8, u8>;
type TypedIEEE80211TLV<Payload> = TLV<u8, u8, IEEE80211TLVType, Payload>;

macro_rules! tlvs {
    (
        $(
            #[$meta_var:meta]
        )*
        pub enum $struct_name:ident <$lt:lifetime $(, $generic:ident: $($trait_bound:path),*  = $default:ty)*> {
            $(
                $(
                    #[$sub_meta_var:meta]
                )*
                $tlv_type_name:ident : $value:expr => $tlv_type:ty
            ),*
        }
    ) => {
        serializable_enum! {
            #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
            pub enum IEEE80211TLVType: u8{
                $(
                    $tlv_type_name => $value,
                )*
                #[default]
                Reserved => 0xe3
            }
        }
        $(
            #[$meta_var]
        )*
        pub enum $struct_name <$lt, $($generic = $default),*> {
            $(
                $(
                    #[$sub_meta_var]
                )*
                $tlv_type_name($tlv_type),
            )*
            Unknown(RawIEEE80211TLV<$lt>)
        }
        impl<$lt $(, $generic: $($trait_bound + )*)*> $struct_name<$lt $(, $generic)*> {
            pub const fn get_tlv_type(&self) -> IEEE80211TLVType {
                match self {
                    $(
                        Self::$tlv_type_name(_) => IEEE80211TLVType::$tlv_type_name,
                    )*
                    Self::Unknown(raw_tlv) => IEEE80211TLVType::Unknown(raw_tlv.tlv_type)
                }
            }
        }
        impl<$lt $(, $generic: $($trait_bound + )*)*> ::scroll::ctx::MeasureWith<()> for $struct_name<$lt $(, $generic)*> {
            fn measure_with(&self, ctx: &()) -> usize {
                2 + match self {
                    $(
                        Self::$tlv_type_name(tlv) => tlv.measure_with(ctx),
                    )*
                    Self::Unknown(raw_tlv) => raw_tlv.slice.len()
                }
            }
        }
        impl<$lt> ::scroll::ctx::TryFromCtx<$lt> for $struct_name<$lt> {
            type Error = ::scroll::Error;
            fn try_from_ctx(from: &$lt [u8], _ctx: ()) -> Result<(Self, usize), ::scroll::Error> {
                let (tlv, len) =
                    <RawIEEE80211TLV<'a> as ::scroll::ctx::TryFromCtx<'a, Endian>>::try_from_ctx(from, Endian::Little)?;
                Ok((
                    match IEEE80211TLVType::from_representation(tlv.tlv_type) {
                        $(
                            IEEE80211TLVType::$tlv_type_name => Self::$tlv_type_name(::scroll::ctx::TryFromCtx::try_from_ctx(tlv.slice, ()).map(|(tlv, _)| tlv)?),
                        )*
                        IEEE80211TLVType::Unknown(_) | IEEE80211TLVType::Reserved => Self::Unknown(tlv)
                    },
                    len
                ))
            }
        }
        impl<$lt $(, $generic: $($trait_bound + )*)*> ::scroll::ctx::TryIntoCtx for $struct_name<$lt $(, $generic)*> {
            type Error = ::scroll::Error;
            fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, ::scroll::Error> {
                match self {
                    $(
                        Self::$tlv_type_name(payload) => buf.pwrite(TypedIEEE80211TLV {
                            tlv_type: IEEE80211TLVType::$tlv_type_name,
                            payload,
                            _phantom: PhantomData
                        }, 0),
                    )*
                    Self::Unknown(payload) => buf.pwrite(payload, 0)
                }

            }
        }
        pub trait ToTLV<$lt $(, $generic: $($trait_bound + )* = $default)*> {
            fn to_tlv(self) -> $struct_name<$lt $(, $generic)*>;
        }
    };
}

tlvs! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// This enum contains all possible elements.   
    pub enum IEEE80211TLV<
        'a,
        RateIterator: IntoIterator<Item = EncodedRate>, Clone = SupportedRatesTLVReadRateIterator<'a>,
        ExtendedRateIterator: IntoIterator<Item = EncodedExtendedRate>, Clone = ExtendedSupportedRatesTLVReadRateIterator<'a>
    > {
        SSID: 0x00 => SSIDTLV<'a>,
        SupportedRates: 0x01 => SupportedRatesTLV<RateIterator>,
        DSSSParameterSet: 0x03 => DSSSParameterSet,
        ExtendedSupportedRates: 0x32 => ExtendedSupportedRatesTLV<ExtendedRateIterator>
    }
}
/// This is an iterator over the elements contained in the body of a frame.
///
/// It's short circuiting.
pub type TLVReadIterator<'a> = ReadIterator<'a, (), IEEE80211TLV<'a>>;
