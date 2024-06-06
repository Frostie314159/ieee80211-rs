//! This module contains types, which are a facade for the actual element types.
//! They exist, to hide the sometimes outrageous amounts of generics, that the actual element types require.

use super::{
    ht_cap_oper::{HTCapabilitiesElement, HTOperationElement},
    rates::{ExtendedSupportedRatesElement, RatesReadIterator, SupportedRatesElement},
    vendor_specific_element::VendorSpecificElement,
    BSSLoadElement, DSSSParameterSetElement, Element, IBSSParameterSetElement, SSIDElement,
};

pub trait ElementTypeRepr {
    type ElementType<'a>: Element;
}
macro_rules! gen_element_type_reprs {
    (
        $(
            $element_type_repr:ident => $element_type:ty
        ),*
    ) => {
        $(
            #[doc = concat!("This is the type state representation for the ", concat!("[", concat!(stringify!($element_type_repr), "Element].")))]
            #[doc = "See the module level documentation for more info."]
            pub struct $element_type_repr;
            impl ElementTypeRepr for $element_type_repr {
                type ElementType<'a> = $element_type;
            }
        )*
    };
}
gen_element_type_reprs! {
    SSIDRepr => SSIDElement<&'a str>,
    SupportedRatesRepr => SupportedRatesElement<RatesReadIterator<'a>>,
    DSSSParameterSetRepr => DSSSParameterSetElement,
    IBSSParameterSetRepr => IBSSParameterSetElement,
    BSSLoadRepr => BSSLoadElement,
    HTCapabilitiesRepr => HTCapabilitiesElement,
    ExtendedSupportedRatesRepr => ExtendedSupportedRatesElement<RatesReadIterator<'a>>,
    HTOperationRepr => HTOperationElement,
    VendorSpecificRepr => VendorSpecificElement<'a>
}
