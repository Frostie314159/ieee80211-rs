use macro_bits::serializable_enum;

serializable_enum! {
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// The piggy-backed control frame of the data frame.
    pub enum DataFrameCF: u8 {
        #[default]
        None => 0b00,
        Ack => 0b01,
        Poll => 0b10,
        AckPoll => 0b11
    }
}

serializable_enum! {
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    /// The subtype of the data frame.
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
        QoSData => 0b1000,
        QoSDataCFAck => 0b1001,
        QoSDataCFPoll => 0b1010,
        QoSDataCFAckCFPoll => 0b1011,
        QoSNull => 0b1100,
        QoSCFPoll => 0b1110,
        QoSCFAckCFPoll => 0b1111
    }
}
impl DataFrameSubtype {
    /// Returns the control frame type piggy-backed on to the data frame.   
    pub const fn data_frame_cf(&self) -> DataFrameCF {
        DataFrameCF::from_bits(self.into_bits() & 0b0011)
    }
    /// Check if the data frame is QoS.
    pub const fn is_qos(&self) -> bool {
        matches!(
            self,
            Self::QoSData
                | Self::QoSDataCFAck
                | Self::QoSDataCFPoll
                | Self::QoSDataCFAckCFPoll
                | Self::QoSNull
                | Self::QoSCFPoll
                | Self::QoSCFAckCFPoll
        )
    }
    /// Check if the data frame has a payload.
    pub const fn has_payload(&self) -> bool {
        matches!(
            self,
            Self::Data
                | Self::DataCFAck
                | Self::DataCFPoll
                | Self::DataCFAckCFPoll
                | Self::QoSData
                | Self::QoSDataCFAck
                | Self::QoSDataCFPoll
                | Self::QoSDataCFAckCFPoll
        )
    }
}
