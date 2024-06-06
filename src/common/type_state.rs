pub trait DSField {
    const TO_DS: bool;
    const FROM_DS: bool;
}
// Indicate not yet decided.
impl DSField for () {
    const TO_DS: bool = false;
    const FROM_DS: bool = false;
}
macro_rules! ds_field_variant {
    ($variant_name:ident, $to_ds:expr, $from_ds:expr) => {
        pub struct $variant_name;
        impl DSField for $variant_name {
            const TO_DS: bool = $to_ds;
            const FROM_DS: bool = $from_ds;
        }
    };
}
ds_field_variant!(NeitherToNorFromDS, false, false);
ds_field_variant!(ToDS, true, false);
ds_field_variant!(FromDS, false, true);
ds_field_variant!(ToAndFromDS, true, true);
