use macro_bits::serializable_enum;

serializable_enum! {
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    /// This is the subtype of a control frame.
    ///
    /// Currently unused.
    pub enum ControlFrameSubtype: u8 {
        TACK => 0b0011,
        BeamformingReportPoll => 0b0100,
        VHTNDPAnnouncement => 0b0101,
        ControlFrameExtension => 0b0110,
        ControlWrapper => 0b0111,
        BlockAckRequest => 0b1000,
        BlockAck => 0b1001,
        PSPoll => 0b1010,
        RTS => 0b1011,
        CTS => 0b1100,
        Ack => 0b1101,
        CFEnd => 0b1110,
        CFEndAck => 0b1111
    }
}
