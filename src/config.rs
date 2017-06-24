#[derive(Deserialize)]
pub struct Config {
    // pub mcnotify: McNotify,
    pub address: Address,
    pub formats: Formats,
}

/*
#[derive(Deserialize)]
pub struct McNotify {
}
*/

#[derive(Deserialize)]
pub struct Address {
    pub hostname: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct Formats {
    /// A notification message sent when the server recovered.
    pub recover_msg: String,

    /// A notification message sent when the server died.
    pub dead_msg: String,

    /// A notification message sent when some player joined the server.
    pub join_msg: String,

    /// A notification message sent when some player left the server.
    pub leave_msg: String,

    pub players_fmt: String,

    pub time_fmt: String,
}
