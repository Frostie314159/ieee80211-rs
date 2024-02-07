#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ChannelSwitchAnnouncement {
    pub no_more_transmissions: bool,
    pub new_channel: u8,
    pub channel_switch_count: u8,
}
impl ChannelSwitchAnnouncement {}
