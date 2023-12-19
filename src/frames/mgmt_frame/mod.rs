use mac_parser::MACAddress;
use macro_bits::serializable_enum;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::{frag_seq_info::FragSeqInfo, frame_control_field::FCFFlags};

use self::{action::ActionFrameBody, beacon::BeaconFrameBody};

pub mod action;
pub mod beacon;

serializable_enum! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum ManagementFrameSubtype: u8 {
        AssociationRequest => 0b0000,
        AssociationResponse => 0b0001,
        ReassociationRequest => 0b0010,
        ReassociationResponse => 0b0011,
        ProbeRequest => 0b0100,
        ProbeResponse => 0b0101,
        TimingAdvertisement => 0b0110,
        Beacon => 0b1000,
        ATIM => 0b1001,
        Disassociation => 0b1010,
        Authentication => 0b1011,
        Deauthentication => 0b1100,
        Action => 0b1101,
        ActionNoAck => 0b1110
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManagementFrameBody<'a> {
    Action(ActionFrameBody<'a>),
    Beacon(BeaconFrameBody<'a>),
    ATIM,
}
impl ManagementFrameBody<'_> {
    pub const fn get_sub_type(&self) -> ManagementFrameSubtype {
        match self {
            Self::Action(_) => ManagementFrameSubtype::Action,
            Self::Beacon(_) => ManagementFrameSubtype::Beacon,
            Self::ATIM => ManagementFrameSubtype::ATIM
        }
    }
    pub const fn length_in_bytes(&self) -> usize {
        match self {
            Self::Action(action) => action.length_in_bytes(),
            Self::Beacon(beacon) => beacon.length_in_bytes(),
            Self::ATIM => 0
        }
    }
}
impl MeasureWith<()> for ManagementFrameBody<'_> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.length_in_bytes()
    }
}
impl<'a> TryFromCtx<'a, ManagementFrameSubtype> for ManagementFrameBody<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(
        from: &'a [u8],
        sub_type: ManagementFrameSubtype,
    ) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;
        Ok((
            match sub_type {
                ManagementFrameSubtype::Action => Self::Action(from.gread(&mut offset)?),
                ManagementFrameSubtype::Beacon => Self::Beacon(from.gread(&mut offset)?),
                ManagementFrameSubtype::ATIM => Self::ATIM,
                _ => {
                    return Err(scroll::Error::BadInput {
                        size: offset,
                        msg: "Management frame subtype not implemented.",
                    })
                }
            },
            offset,
        ))
    }
}
impl TryIntoCtx for ManagementFrameBody<'_> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        match self {
            Self::Action(action_frame_body) => buf.pwrite(action_frame_body, 0),
            Self::Beacon(beacon_frame_body) => buf.pwrite(beacon_frame_body, 0),
            Self::ATIM => Ok(0)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManagementFrame<'a> {
    pub fcf_flags: FCFFlags,

    pub duration: u16,
    pub receiver_address: MACAddress,
    pub transmitter_address: MACAddress,
    pub bssid: MACAddress,
    pub frag_seq_info: FragSeqInfo,
    pub ht_control: Option<[u8; 4]>,
    pub body: ManagementFrameBody<'a>,
}
impl ManagementFrame<'_> {
    pub const fn length_in_bytes(&self) -> usize {
        2 + // Duration
        6 + // Receiver address
        6 + // Transmitter address
        6 + // BSSID
        2 + // Fragement and sequence info
        if self.ht_control.is_some() { 4 } else { 0 } + // HTC
        self.body.length_in_bytes()
    }
}
impl<'a> TryFromCtx<'a, (u8, FCFFlags)> for ManagementFrame<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(
        from: &'a [u8],
        (sub_type, fcf_flags): (u8, FCFFlags),
    ) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let duration = from.gread_with(&mut offset, Endian::Little)?;
        let receiver_address = from.gread(&mut offset)?;
        let transmitter_address = from.gread(&mut offset)?;
        let bssid = from.gread(&mut offset)?;
        let frag_seq_info =
            FragSeqInfo::from_representation(from.gread_with(&mut offset, Endian::Little)?);
        let ht_control = if fcf_flags.htc_plus_order {
            Some(from.gread(&mut offset)?)
        } else {
            None
        };
        let body = from.gread_with(
            &mut offset,
            ManagementFrameSubtype::from_representation(sub_type),
        )?;

        Ok((
            Self {
                fcf_flags,
                duration,
                receiver_address,
                transmitter_address,
                bssid,
                frag_seq_info,
                ht_control,
                body,
            },
            offset,
        ))
    }
}
impl TryIntoCtx for ManagementFrame<'_> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(self.duration, &mut offset, Endian::Little)?;
        buf.gwrite(self.receiver_address, &mut offset)?;
        buf.gwrite(self.transmitter_address, &mut offset)?;
        buf.gwrite(self.bssid, &mut offset)?;
        buf.gwrite_with(
            self.frag_seq_info.to_representation(),
            &mut offset,
            Endian::Little,
        )?;
        if let Some(ht_control) = self.ht_control {
            buf.gwrite(ht_control, &mut offset)?;
        }
        buf.gwrite(self.body, &mut offset)?;
        Ok(offset)
    }
}
