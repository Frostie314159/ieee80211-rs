use std::marker::PhantomData;

use ieee80211::{
    aid,
    elements::tim::{ConstBitmap, TIMElement},
    tim_bitmap,
};

use crate::gen_element_rw_test;

const EXPECTED_TIM_ELEMENT: TIMElement<ConstBitmap> = TIMElement {
    dtim_count: 2,
    dtim_period: 3,
    bitmap: Some(tim_bitmap![0, 12, 13]),
    _phantom: PhantomData,
};
const EXPECTED_TIM_ELEMENT_BYTES: &[u8] = &[0x02, 0x03, 0x01, 0x01, 0x30];

gen_element_rw_test!(
    test_tim_element_rw,
    TIMElement,
    EXPECTED_TIM_ELEMENT,
    EXPECTED_TIM_ELEMENT_BYTES
);

#[test]
fn test_tim_aid_decode() {
    assert!(EXPECTED_TIM_ELEMENT
        .bitmap
        .unwrap()
        .aid_iter()
        .unwrap()
        .into_iter()
        .eq([aid!(12), aid!(13)]));
}
