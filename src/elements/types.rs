use super::{
    rates::{RatesReadIterator, SupportedRatesElement},
    Element, SSIDElement,
};

pub trait ElementTypeRepr {
    type ElementType<'a>: Element<'a>;
}
macro_rules! gen_element_type_reprs {
    (
        $(
            $element_type_repr:ident => $element_type:ty
        ),*
    ) => {
        $(
            pub struct $element_type_repr;
            impl ElementTypeRepr for $element_type_repr {
                type ElementType<'a> = $element_type;
            }
        )*
    };
}
gen_element_type_reprs! {
    SSID => SSIDElement<'a>,
    SupportedRates => SupportedRatesElement<RatesReadIterator<'a>>
}
