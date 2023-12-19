use macro_bits::{bit, bitfield, serializable_enum};

serializable_enum! {
    #[derive(Debug, Default, Clone, Copy, PartialEq)]
    pub enum FrameTypes: u8 {
        #[default]
        AssociationRequest => 0x00,
        Data => 0x02,
        AssociationResponse => 0x04,
        ReassociationRequest => 0x08,
        ReassociationResponse => 0x0C,
        ProbeRequest => 0x10,
        NullFunction => 0x12,
        ProbeResponse => 0x14,
        VHTNDPAnnouncement => 0x15,
        MeasurementPilot => 0x18,
        Beacon => 0x20,
        BlockACKReqeust => 0x21,
        QOSData => 0x22,
        AnnouncementTrafficIndication => 0x24,
        BlockACK => 0x25,
        Disassociation => 0x28,
        Authentication => 0x2C,
        RequestToSend => 0x2D,
        Deauthentication => 0x30,
        ClearToSend => 0x31,
        QOSNullFunction => 0x32,
        ACK => 0x35,
        Action => 0x34,
        ActionNoACK => 0x38,
        ControlFrameEnd => 0x39
    }
}
serializable_enum! {
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
    pub enum FrameType: u8 {
        #[default]
        Management => 0b00,
        Control => 0b01,
        Data => 0b10
    }
}
bitfield! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct FCFFlags: u8 {
        pub to_ds: bool => bit!(0),
        pub from_ds: bool => bit!(1),
        pub more_fragments: bool => bit!(2),
        pub retry: bool => bit!(3),
        pub pwr_mgt: bool => bit!(4),
        pub more_data: bool => bit!(5),
        pub protected: bool => bit!(6),
        pub htc_plus_order: bool => bit!(7)
    }
}
bitfield! {
    #[derive(Debug, Default, Clone, Copy, PartialEq)]
    pub struct FrameControlField: u16 {
        pub version: u8 => bit!(0,1),
        pub frame_type: FrameType => bit!(2,3),
        pub frame_sub_type: u8 => bit!(4,5,6,7),
        pub flags: FCFFlags => 0xff00
    }
}
