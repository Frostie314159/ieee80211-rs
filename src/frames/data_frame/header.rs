use mac_parser::MACAddress;
use macro_bits::bit;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

use crate::common::*;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// A generic data frame header.
///
/// The address fields are unnamed, since their meaning is context dependent.
/// To access them use the provided methods.
pub struct DataFrameHeader {
    // Shared across all headers.
    /// Subtype of the frame.
    pub subtype: DataFrameSubtype,
    /// Flags as specified in the [Frame Control Field](crate::common::FrameControlField).
    pub fcf_flags: FCFFlags,

    // Actual header from here.
    pub duration: u16,
    /// First address.
    pub address_1: MACAddress,
    /// Second address.
    pub address_2: MACAddress,
    /// Third address.
    pub address_3: MACAddress,
    /// Sequence control
    pub sequence_control: SequenceControl,
    /// Potentially fourth address.
    pub address_4: Option<MACAddress>,
    pub qos: Option<[u8; 2]>,
    pub ht_control: Option<[u8; 4]>,
}
impl DataFrameHeader {
    /// Generate the [FrameControlField] from the header.
    pub const fn get_fcf(&self) -> FrameControlField {
        FrameControlField::new()
            .with_frame_type(FrameType::Data(self.subtype))
            .with_flags(self.fcf_flags)
    }
    /// The total length in bytes of the header.
    ///
    /// This can be used in const contexts.
    pub const fn length_in_bytes(&self) -> usize {
        let mut size = 2 + 6 + 6 + 6 + 2;
        if self.address_4.is_some() {
            size += 6;
        }
        if self.qos.is_some() {
            size += 2;
        }
        if self.ht_control.is_some() {
            size += 4;
        }
        size
    }
    /// Check if the data frame is an A-MSDU.
    pub const fn is_amsdu(&self) -> bool {
        if let Some(qos) = self.qos {
            qos[0] & bit!(7) != 0 && self.subtype.has_payload()
        } else {
            false
        }
    }
    /// Check if no control frame is encapsulated.
    ///
    /// # Returns
    /// `true` If the subtype is one of the following: [Data](DataFrameSubtype::Data), [Null](DataFrameSubtype::Null), [QoSData](DataFrameSubtype::QoSData) or [QoSNull](DataFrameSubtype::QoSNull).
    /// `false` In all other cases.
    pub const fn is_no_cf(&self) -> bool {
        matches!(
            self.subtype,
            DataFrameSubtype::Data
                | DataFrameSubtype::Null
                | DataFrameSubtype::QoSData
                | DataFrameSubtype::QoSNull
        )
    }
    /// Check if an Ack control frame is encapsulated.
    ///
    /// # Returns
    /// `true` If the subtype is one of the following: [DataCFAck](DataFrameSubtype::DataCFAck), [CFAck](DataFrameSubtype::CFAck) or [QoSDataCFAck](DataFrameSubtype::QoSDataCFAck).
    /// `false` In all other cases.
    pub const fn is_cf_ack(&self) -> bool {
        matches!(
            self.subtype,
            DataFrameSubtype::DataCFAck | DataFrameSubtype::CFAck | DataFrameSubtype::QoSDataCFAck
        )
    }
    /// Check if a Poll control frame is encapsulated.
    ///
    /// # Returns
    /// `true` If the subtype is one of the following: [DataCFPoll](DataFrameSubtype::DataCFPoll), [CFPoll](DataFrameSubtype::CFPoll), [QoSDataCFPoll](DataFrameSubtype::QoSDataCFPoll) or [QoSCFPoll](DataFrameSubtype::QoSCFPoll).
    /// `false` In all other cases.
    pub const fn is_cf_poll(&self) -> bool {
        matches!(
            self.subtype,
            DataFrameSubtype::DataCFPoll
                | DataFrameSubtype::CFPoll
                | DataFrameSubtype::QoSDataCFPoll
                | DataFrameSubtype::QoSCFPoll
        )
    }
    /// Check if an Ack/Poll control frame is encapsulated.
    ///
    /// # Returns
    /// `true` If the subtype is one of the following: [DataCFAckCFPoll](DataFrameSubtype::DataCFAckCFPoll), [CFAckCFPoll](DataFrameSubtype::CFAckCFPoll), [QoSDataCFAckCFPoll](DataFrameSubtype::QoSDataCFAckCFPoll) or [QoSCFAckCFPoll](DataFrameSubtype::QoSCFAckCFPoll).
    /// `false` In all other cases.
    pub const fn is_cf_ack_poll(&self) -> bool {
        matches!(
            self.subtype,
            DataFrameSubtype::DataCFAckCFPoll
                | DataFrameSubtype::CFAckCFPoll
                | DataFrameSubtype::QoSDataCFAckCFPoll
                | DataFrameSubtype::QoSCFAckCFPoll
        )
    }

    /// Returns a reference to the receiver address.
    ///
    /// This will always return the first address.
    pub const fn receiver_address(&self) -> &MACAddress {
        &self.address_1
    }

    /// Returns a reference to the transmitter address.
    ///
    /// This will always return the second address.
    pub const fn transmitter_address(&self) -> &MACAddress {
        &self.address_2
    }

    /// Returns an optional reference to the destination address.
    ///
    /// # Mapping
    /// To DS | From DS | Is A-MSDU | Address
    /// -- | -- | -- | --
    /// No | * | * | One
    /// \* | * | No | Three
    pub const fn destination_address(&self) -> Option<&MACAddress> {
        if !self.fcf_flags.to_ds() {
            Some(&self.address_1)
        } else if !self.is_amsdu() {
            Some(&self.address_3)
        } else {
            None
        }
    }
    /// Returns an optional mutable reference to the destination address.
    ///
    /// The mapping is the same as [`Self::destination_address()`].
    pub fn destination_address_mut(&mut self) -> Option<&mut MACAddress> {
        if !self.fcf_flags.to_ds() {
            Some(&mut self.address_1)
        } else if !self.is_amsdu() {
            Some(&mut self.address_3)
        } else {
            None
        }
    }

    /// Returns an optional reference to the source_address.
    ///
    /// # Mapping
    /// To DS | From DS | Is A-MSDU | Address
    /// -- | -- | -- | --
    /// \* | No | * | Two
    /// No | Yes | No | Three
    /// Yes | Yes | No | Four
    pub const fn source_address(&self) -> Option<&MACAddress> {
        if !self.fcf_flags.from_ds() {
            Some(&self.address_2)
        } else if !self.fcf_flags.to_ds() && self.fcf_flags.from_ds() && !self.is_amsdu() {
            Some(&self.address_3)
        } else if self.fcf_flags.to_ds() && self.fcf_flags.from_ds() && !self.is_amsdu() {
            self.address_4.as_ref()
        } else {
            None
        }
    }
    /// Returns an optional mutable reference to the source address.
    ///
    /// The mapping is the same as [`Self::source_address()`].
    pub fn source_address_mut(&mut self) -> Option<&mut MACAddress> {
        if !self.fcf_flags.from_ds() {
            Some(&mut self.address_2)
        } else if !self.fcf_flags.to_ds() && self.fcf_flags.from_ds() && !self.is_amsdu() {
            Some(&mut self.address_3)
        } else if self.fcf_flags.to_ds() && self.fcf_flags.from_ds() && !self.is_amsdu() {
            self.address_4.as_mut()
        } else {
            None
        }
    }

    /// Returns an optional reference to the bssid.
    ///
    /// # Mapping
    /// To DS | From DS | Is A-MSDU | Address
    /// -- | -- | -- | --
    /// No | No | * | Three
    /// \* | Yes | * | Two
    /// Yes | * | * | One
    pub const fn bssid(&self) -> Option<&MACAddress> {
        if !self.fcf_flags.to_ds() && !self.fcf_flags.from_ds() {
            Some(&self.address_3)
        } else if self.fcf_flags.from_ds() {
            Some(&self.address_2)
        } else if self.fcf_flags.to_ds() {
            Some(&self.address_1)
        } else {
            None
        }
    }
    /// Returns an optional mutable reference to the bssid.
    ///
    /// The mapping is the same as [`Self::bssid()`].
    pub fn bssid_mut(&mut self) -> Option<&mut MACAddress> {
        if (!self.fcf_flags.to_ds() && !self.fcf_flags.from_ds()) || self.is_amsdu() {
            Some(&mut self.address_3)
        } else if self.fcf_flags.from_ds() {
            Some(&mut self.address_2)
        } else if self.fcf_flags.to_ds() {
            Some(&mut self.address_1)
        } else {
            None
        }
    }
}
impl MeasureWith<()> for DataFrameHeader {
    fn measure_with(&self, _ctx: &()) -> usize {
        self.length_in_bytes()
    }
}
impl TryFromCtx<'_> for DataFrameHeader {
    type Error = scroll::Error;
    fn try_from_ctx(from: &'_ [u8], _ctx: ()) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let fcf = FrameControlField::from_bits(from.gread_with(&mut offset, Endian::Little)?);
        let FrameType::Data(subtype) = fcf.frame_type() else {
            return Err(scroll::Error::BadInput {
                size: offset,
                msg: "The frame type in the FCF wasn't data.",
            });
        };
        let duration = from.gread(&mut offset)?;
        let address_1 = from.gread(&mut offset)?;
        let address_2 = from.gread(&mut offset)?;
        let address_3 = from.gread(&mut offset)?;
        let frag_seq_info = SequenceControl::from_bits(from.gread(&mut offset)?);
        let address_4 = if fcf.flags().to_ds() && fcf.flags().from_ds() {
            Some(from.gread(&mut offset)?)
        } else {
            None
        };
        let qos = if subtype.is_qos() {
            Some(from.gread(&mut offset)?)
        } else {
            None
        };
        let ht_control = if fcf.flags().order() {
            Some(from.gread(&mut offset)?)
        } else {
            None
        };

        Ok((
            Self {
                subtype,
                fcf_flags: fcf.flags(),
                duration,
                address_1,
                address_2,
                address_3,
                sequence_control: frag_seq_info,
                address_4,
                qos,
                ht_control,
            },
            offset,
        ))
    }
}
impl TryIntoCtx for DataFrameHeader {
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], _ctx: ()) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(self.duration, &mut offset, Endian::Little)?;
        buf.gwrite(self.address_1, &mut offset)?;
        buf.gwrite(self.address_2, &mut offset)?;
        buf.gwrite(self.address_3, &mut offset)?;
        buf.gwrite_with(
            self.sequence_control.into_bits(),
            &mut offset,
            Endian::Little,
        )?;
        if let Some(address_4) = self.address_4 {
            buf.gwrite(address_4, &mut offset)?;
        }
        if let Some(qos) = self.qos {
            buf.gwrite(qos, &mut offset)?;
        }
        if let Some(ht_control) = self.ht_control {
            buf.gwrite(ht_control, &mut offset)?;
        }

        Ok(offset)
    }
}
