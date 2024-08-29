use mac_parser::MACAddress;
use scroll::{
    ctx::{TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::common::{FCFFlags, SequenceControl};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// A management frame header.
pub struct ManagementFrameHeader {
    /// The flags from the [FrameControlField](crate::common::FrameControlField).
    pub fcf_flags: FCFFlags,
    pub duration: u16,
    pub receiver_address: MACAddress,
    pub transmitter_address: MACAddress,
    pub bssid: MACAddress,
    pub sequence_control: SequenceControl,
    pub ht_control: Option<[u8; 4]>,
}
impl ManagementFrameHeader {
    pub const fn length_in_bytes(&self) -> usize {
        let mut size = 2 + 6 + 6 + 6 + 2;
        if self.ht_control.is_some() {
            size += 4;
        }
        size
    }
}
impl TryFromCtx<'_, FCFFlags> for ManagementFrameHeader {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'_ [u8], fcf_flags: FCFFlags) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let duration = from.gread_with(&mut offset, Endian::Little)?;
        let receiver_address = from.gread(&mut offset)?;
        let transmitter_address = from.gread(&mut offset)?;
        let bssid = from.gread(&mut offset)?;
        let frag_seq_info =
            SequenceControl::from_bits(from.gread_with(&mut offset, Endian::Little)?);
        let ht_control = if fcf_flags.order() {
            Some(from.gread(&mut offset)?)
        } else {
            None
        };
        Ok((
            Self {
                fcf_flags,
                duration,
                receiver_address,
                transmitter_address,
                bssid,
                sequence_control: frag_seq_info,
                ht_control,
            },
            offset,
        ))
    }
}
impl TryIntoCtx for ManagementFrameHeader {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(self.duration, &mut offset, Endian::Little)?;
        buf.gwrite(self.receiver_address, &mut offset)?;
        buf.gwrite(self.transmitter_address, &mut offset)?;
        buf.gwrite(self.bssid, &mut offset)?;
        buf.gwrite_with(
            self.sequence_control.into_bits(),
            &mut offset,
            Endian::Little,
        )?;
        if let Some(ht_control) = self.ht_control {
            buf.gwrite(ht_control, &mut offset)?;
        }

        Ok(offset)
    }
}
