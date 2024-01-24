use macro_bits::serializable_enum;

serializable_enum! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    /// The subtype of a management frame.
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
