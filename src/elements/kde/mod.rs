#[macro_export]
macro_rules! define_kde {
    ($selector:expr, $kde_type:ident) => {
        impl crate::elements::Element for $kde_type {
            const ELEMENT_ID: crate::elements::ElementID = crate::elements::ElementID::VendorSpecific {
                prefix: &[0x00, 0x0f, 0xac, $selector],
            };
            type ReadType<'a> = $kde_type;
        }
    };
}
