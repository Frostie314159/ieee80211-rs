use mac_parser::MACAddress;
use macro_bits::{bit, serializable_enum};
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

use crate::{frag_seq_info::FragSeqInfo, frame_control_field::FCFFlags};

use self::
    amsdu::AMSDUPayload
;

pub mod amsdu;
pub mod builder;

serializable_enum! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub enum DataFrameSubtype: u8 {
        #[default]
        Data => 0b0000,
        DataCFAck => 0b0001,
        DataCFPoll => 0b0010,
        DataCFAckCFPoll => 0b0011,
        Null => 0b0100,
        CFAck => 0b0101,
        CFPoll => 0b0110,
        CFAckCFPoll => 0b0111,
        QOSData => 0b1000,
        QOSDataCFAck => 0b1001,
        QOSDataCFPoll => 0b1010,
        QOSDataCFAckCFPoll => 0b1011,
        QOSNull => 0b1100,
        QOSCFPoll => 0b1110,
        QOSCFAckCFPoll => 0b1111
    }
}
impl DataFrameSubtype {
    pub const fn is_qos(&self) -> bool {
        matches!(
            self,
            Self::QOSData
                | Self::QOSDataCFAck
                | Self::QOSDataCFPoll
                | Self::QOSDataCFAckCFPoll
                | Self::QOSNull
                | Self::QOSCFPoll
                | Self::QOSCFAckCFPoll
        )
    }
    pub const fn has_payload(&self) -> bool {
        matches!(
            self,
            Self::Data
                | Self::DataCFAck
                | Self::DataCFPoll
                | Self::DataCFAckCFPoll
                | Self::QOSData
                | Self::QOSDataCFAck
                | Self::QOSDataCFPoll
                | Self::QOSDataCFAckCFPoll
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataFramePayload<'a> {
    Single(&'a [u8]),
    AMSDU(AMSDUPayload<'a>),
}
impl DataFramePayload<'_> {
    pub const fn length_in_bytes(&self) -> usize {
        match self {
            Self::Single(slice) => slice.len(),
            Self::AMSDU(amsdu_payload) => amsdu_payload.length_in_bytes()
        }
    }
}
impl TryIntoCtx for DataFramePayload<'_> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        match self {
            DataFramePayload::Single(slice) => buf.pwrite(slice, 0),
            DataFramePayload::AMSDU(amsdu_payload) => {
                let mut offset = 0;
                for amsdu_sub_frame in amsdu_payload.sub_frames {
                    buf.gwrite(*amsdu_sub_frame, &mut offset)?;
                }
                Ok(offset)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataFrame<'a> {
    pub sub_type: DataFrameSubtype,
    pub fcf_flags: FCFFlags,

    pub duration: u16,
    pub address_1: MACAddress,
    pub address_2: MACAddress,
    pub address_3: MACAddress,
    pub frag_seq_info: FragSeqInfo,
    pub address_4: Option<MACAddress>,
    pub qos: Option<[u8; 2]>,
    pub ht_control: Option<[u8; 4]>,
    payload: Option<DataFramePayload<'a>>,
}
impl DataFrame<'_> {
    const fn is_amsdu(&self) -> bool {
        if let Some(qos) = self.qos {
            qos[0] & bit!(7) != 0 && self.sub_type.has_payload()
        } else {
            false
        }
    }
    pub const fn get_sub_type(&self) -> DataFrameSubtype {
        self.sub_type
    }

    pub const fn receiver_address(&self) -> &MACAddress {
        &self.address_1
    }

    pub const fn transmitter_address(&self) -> &MACAddress {
        &self.address_2
    }

    pub const fn destination_address(&self) -> Option<&MACAddress> {
        if !self.fcf_flags.to_ds {
            Some(&self.address_1)
        } else if !self.is_amsdu() {
            Some(&self.address_3)
        } else {
            None
        }
    }
    pub fn destination_address_mut(&mut self) -> Option<&mut MACAddress> {
        if !self.fcf_flags.to_ds {
            Some(&mut self.address_1)
        } else if !self.is_amsdu() {
            Some(&mut self.address_3)
        } else {
            None
        }
    }

    pub const fn source_address(&self) -> Option<&MACAddress> {
        if !self.fcf_flags.from_ds {
            Some(&self.address_2)
        } else if !self.fcf_flags.to_ds && self.fcf_flags.from_ds && !self.is_amsdu() {
            Some(&self.address_3)
        } else if self.fcf_flags.to_ds && self.fcf_flags.from_ds && !self.is_amsdu() {
            self.address_4.as_ref()
        } else {
            None
        }
    }
    pub fn source_address_mut(&mut self) -> Option<&mut MACAddress> {
        if !self.fcf_flags.from_ds {
            Some(&mut self.address_2)
        } else if !self.fcf_flags.to_ds && self.fcf_flags.from_ds && !self.is_amsdu() {
            Some(&mut self.address_3)
        } else if self.fcf_flags.to_ds && self.fcf_flags.from_ds && !self.is_amsdu() {
            self.address_4.as_mut()
        } else {
            None
        }
    }

    pub const fn bssid(&self) -> Option<&MACAddress> {
        if (!self.fcf_flags.to_ds && !self.fcf_flags.from_ds) || self.is_amsdu() {
            Some(&self.address_3)
        } else if self.fcf_flags.from_ds {
            Some(&self.address_2)
        } else if self.fcf_flags.to_ds {
            Some(&self.address_1)
        } else {
            None
        }
    }
    pub fn bssid_mut(&mut self) -> Option<&mut MACAddress> {
        if (!self.fcf_flags.to_ds && !self.fcf_flags.from_ds) || self.is_amsdu() {
            Some(&mut self.address_3)
        } else if self.fcf_flags.from_ds {
            Some(&mut self.address_2)
        } else if self.fcf_flags.to_ds {
            Some(&mut self.address_1)
        } else {
            None
        }
    }

    pub const fn length_in_bytes(&self) -> usize {
        2 + // Duration
        6 + // Address 1
        6 + // Address 2
        6 + // Address 3
        2 + // FragSeqInfo
        if self.address_4.is_some() { 6 } else { 0 } + // Address 4
        if self.qos.is_some() { 2 } else { 0 } + // QoS
        if let Some(payload) = self.payload { payload.length_in_bytes() } else { 0 }
        // Payload
    }
}
impl MeasureWith<()> for DataFrame<'_> {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.length_in_bytes()
    }
}
impl<'a> TryFromCtx<'a, (u8, FCFFlags)> for DataFrame<'a> {
    type Error = scroll::Error;
    fn try_from_ctx(
        from: &'a [u8],
        (sub_type, fcf_flags): (u8, FCFFlags),
    ) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let sub_type = DataFrameSubtype::from_representation(sub_type);
        let duration = from.gread(&mut offset)?;
        let address_1 = from.gread(&mut offset)?;
        let address_2 = from.gread(&mut offset)?;
        let address_3 = from.gread(&mut offset)?;
        let frag_seq_info = FragSeqInfo::from_representation(from.gread(&mut offset)?);
        let address_4 = if fcf_flags.to_ds && fcf_flags.from_ds {
            Some(from.gread(&mut offset)?)
        } else {
            None
        };
        let qos = if sub_type.is_qos() {
            Some(from.gread(&mut offset)?)
        } else {
            None
        };
        let ht_control = if fcf_flags.htc_plus_order {
            Some(from.gread(&mut offset)?)
        } else {
            None
        };
        let payload = if sub_type.has_payload() {
            let len = from.len() - offset;
            Some(DataFramePayload::Single(from.gread_with(&mut offset, len)?))
        } else {
            None
        };
        Ok((
            Self {
                sub_type,
                fcf_flags,
                duration,
                address_1,
                address_2,
                address_3,
                frag_seq_info,
                address_4,
                qos,
                ht_control,
                payload,
            },
            offset,
        ))
    }
}
impl TryIntoCtx for DataFrame<'_> {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite(self.duration, &mut offset)?;
        buf.gwrite(self.address_1, &mut offset)?;
        buf.gwrite(self.address_2, &mut offset)?;
        buf.gwrite(self.address_3, &mut offset)?;
        buf.gwrite(self.frag_seq_info.to_representation(), &mut offset)?;
        if let Some(address_4) = self.address_4 {
            buf.gwrite(address_4, &mut offset)?;
        }
        if let Some(qos) = self.qos {
            buf.gwrite(qos, &mut offset)?;
        }
        if let Some(ht_control) = self.ht_control {
            buf.gwrite(ht_control, &mut offset)?;
        }
        if let Some(payload) = self.payload {
            buf.gwrite(payload, &mut offset)?;
        }
        Ok(offset)
    }
}
