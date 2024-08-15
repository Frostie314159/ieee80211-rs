use bitfield_struct::bitfield;

#[bitfield(u8)]
#[derive(PartialEq, Eq, Hash)]
/// The bitmap control field of a [TIMElement].
pub struct TIMBitmapControl {
    pub traffic_indicator: bool,
    #[bits(7)]
    pub bitmap_offset: u8
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TIMElement {
    pub dtim_count: u8,
    pub dtim_period: u8,
    pub bitmap_control: Option<TIMBitmapControl>
}
