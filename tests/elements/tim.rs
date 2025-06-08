use std::marker::PhantomData;

use ieee80211::{
    aid,
    elements::tim::{ConstBitmap, TIMElement},
    tim_bitmap,
};

use crate::roundtrip_test;

const EXPECTED_TIM_ELEMENT: TIMElement<ConstBitmap> = TIMElement {
    dtim_count: 2,
    dtim_period: 3,
    bitmap: Some(tim_bitmap![0, 12, 13]),
    _phantom: PhantomData,
};
const EXPECTED_TIM_ELEMENT_BYTES: &[u8] = &[0x02, 0x03, 0x01, 0x01, 0x30];

roundtrip_test!(
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
        .eq([aid!(12), aid!(13)]));
}
const EMPTY_TIM_BYTES: &[u8] = &[0x02, 0x03];
roundtrip_test!(
    test_empty_tim_element_rw,
    TIMElement,
    TIMElement {
        dtim_count: 2,
        dtim_period: 3,
        bitmap: TIMElement::NO_TIM_BITMAP,
        _phantom: PhantomData,
    },
    EMPTY_TIM_BYTES
);
