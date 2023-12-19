use macro_bits::{bit, bitfield};

bitfield! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct FragSeqInfo: u16 {
        pub fragment_number: u8 => bit!(0,1,2,3),
        pub sequence_number: u16 => bit!(4,5,6,7,8,9,10,11,12,13,14,15)
    }
}
