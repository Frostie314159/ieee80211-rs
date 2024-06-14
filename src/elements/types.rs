//! This module contains types, which are a facade for the actual element types.
//! They exist, to hide the sometimes outrageous amounts of generics, that the actual element types require.

use crate::common::ieee80211_list::IEEE80211ReadList;

use super::{
    ht_cap_oper::{HTCapabilitiesElement, HTOperationElement},
    rates::{ExtendedSupportedRatesElement, RatesReadIterator, SupportedRatesElement},
    rsn::{IEEE80211AKMType, IEEE80211CipherSuiteSelector, RSNElement, IEEE80211PMKID},
    BSSLoadElement, DSSSParameterSetElement, Element, IBSSParameterSetElement, SSIDElement,
    VendorSpecificElement,
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
    RSNRepr => RSNElement<
        IEEE80211ReadList<'a, IEEE80211CipherSuiteSelector, u16, 4>,
        IEEE80211ReadList<'a, IEEE80211AKMType, u16, 4>,
        IEEE80211ReadList<'a, IEEE80211PMKID, u16, 16>
    >,
    VendorSpecificRepr => VendorSpecificElement<'a>
}
