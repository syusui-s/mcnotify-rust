#[derive(Deserialize)]
pub struct Config {
    pub mcnotify: McNotify,
    pub address: Address,
    pub formats: Formats,
    pub twitter: TwitterConfig,
}

#[derive(Deserialize)]
pub struct McNotify {
    pub check_interval: u16,
}

#[derive(Deserialize)]
pub struct Address {
    pub hostname: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct Formats {
    /// A notification message sent when the server recovered.
    pub recover_msg: String,

    /// A notification message sent when the server down.
    pub down_msg: String,

    /// A notification message sent when some player joined the server.
    pub join_fmt: String,

    /// A notification message sent when some player left the server.
    pub leave_fmt: String,

    pub players_fmt: String,

    pub time_fmt: String,
}

#[derive(Deserialize)]
pub struct TwitterConfig {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub access_key: String,
    pub access_secret: String,
}
