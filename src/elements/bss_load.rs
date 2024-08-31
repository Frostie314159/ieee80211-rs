use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use super::{Element, ElementID};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// The BSS Load element contains information on the current STA population and traffic levels in the BSS.
pub struct BSSLoadElement {
    /// The number of STAs associated with the BSS.
    pub station_count: u16,
    /// The Channel Utilization field is defined as the percentage of time, linearly scaled with 255 representing 100%,
    /// that the AP sensed the medium was busy, as indicated by either the physical or virtual carrier sense (CS) mechanism.
    pub channel_utilization: u8,
    /// The Available Admission Capacity field contains an unsigned integer that specifies the remaining amount of
    /// medium time available via explicit admission control, in units of 32 µs/s.
    pub available_admission_capacity: u16,
}
impl MeasureWith<()> for BSSLoadElement {
    fn measure_with(&self, _ctx: &()) -> usize {
        5
    }
}
impl TryFromCtx<'_> for BSSLoadElement {
    type Error = scroll::Error;
    fn try_from_ctx(from: &[u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let station_count = from.gread_with(&mut offset, Endian::Little)?;
        let channel_utilization = from.gread(&mut offset)?;
        let available_admission_capacity = from.gread_with(&mut offset, Endian::Little)?;

        Ok((
            Self {
                station_count,
                channel_utilization,
                available_admission_capacity,
            },
            offset,
        ))
    }
}
impl TryIntoCtx for BSSLoadElement {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(self.station_count, &mut offset, Endian::Little)?;
        buf.gwrite(self.channel_utilization, &mut offset)?;
        buf.gwrite_with(
            self.available_admission_capacity,
            &mut offset,
            Endian::Little,
        )?;

        Ok(offset)
    }
}

impl Element for BSSLoadElement {
    const ELEMENT_ID: ElementID = ElementID::Id(0x0b);
    type ReadType<'a> = BSSLoadElement;
}
