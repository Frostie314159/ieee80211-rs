use macro_bits::serializable_enum;

serializable_enum! {
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum ManagementFrameSubtype: u8 {
        AssociationRequest => 0b0000,
        AssociationResponse => 0b0001,
        ProbeRequest => 0b0100,
        ProbeResponse => 0b0101,
        Beacon => 0b1000,
        ATIM => 0b1001,
        Disassociation => 0b1010,
        Authentication => 0b1011,
        Deauthentication => 0b1100,
        Action => 0b1101,
        ActionNoACK => 0b1110
    }
}
