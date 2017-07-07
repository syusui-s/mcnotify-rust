pub enum Message<'a> {
    PlayerChange {
        online_count: u32,
        current_players: &'a Vec<String>,
        joined_players: &'a Vec<String>,
        left_players: &'a Vec<String>,
    },
    Recover {
        online_count: u32,
        current_players: &'a Vec<String>,
    },
    Down,
}
