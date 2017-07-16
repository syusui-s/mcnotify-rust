use minecraft::{client, packet};
use minecraft::json_data::status::Player;
use status_checker::Status::*;

/// represents a server state.
#[derive(Clone)]
pub enum Status {
    Available {
        online_count: u32,
        current_players: Vec<String>,
        joined_players: Vec<String>,
        left_players: Vec<String>,
    },
    Unavailable {
        reason: String,
    },
}

impl Status {
    pub fn unavailable(reason: &str) -> Self {
        Unavailable { reason: reason.to_owned() }
    }

    pub fn current_players(&self) -> Option<&Vec<String>> {
        match self {
            &Available { ref current_players, .. } => Some(current_players),
            &Unavailable { .. } => None,
        }
    }

    pub fn joined_players(&self) -> Option<&Vec<String>> {
        match self {
            &Available { ref joined_players, .. } => Some(joined_players),
            &Unavailable { .. } => None,
        }
    }

    pub fn left_players(&self) -> Option<&Vec<String>> {
        match self {
            &Available { ref left_players, .. } => Some(left_players),
            &Unavailable { .. } => None,
        }
    }

    pub fn reason(&self) -> Option<&String> {
        match self {
            &Available { .. } => None,
            &Unavailable { ref reason } => Some(reason),
        }
    }
}

pub struct StatusChecker {
    hostname: String,
    port: u16,
    last_players: Vec<Player>,
}

impl StatusChecker {
     pub fn new(hostname: &str, port: u16) -> Self {
        Self { hostname: hostname.to_owned(), port, last_players: vec![] }
    }

    pub fn check_status(&mut self) -> Status {
        let address = client::ServerAddr::new(&self.hostname, self.port);

        // get status
        let mut cli = match client::Client::connect(address) {
            Ok(cli) => cli,
            Err(_) => return Status::unavailable(&format!("Couldn't connect to {}:{}", self.hostname, self.port)),
        };

        match cli.handshake(packet::NextState::Status) {
            Ok(_) => { },
            Err(_) => return Status::unavailable(&format!("Couldn't handshake with {}:{}", self.hostname, self.port)),
        }

        let status = match cli.list() {
            Ok(res) => res,
            Err(e) => return Status::unavailable(&format!("List Request was failed : {:?}", e)),
        };

        // build information
        let player_info     = status.players;
        let online_count    = player_info.online;
        let current_players = player_info.sample.unwrap_or_else(|| vec![]);
        let last_players    = self.last_players.clone();

        // Current - Last = Joined
        let joined_players = current_players.iter().filter(|ref current| {
            last_players.iter().all(|ref last| last.id != current.id)
        });

        // Last - Current = Left
        let left_players = last_players.iter().filter(|ref last| {
            current_players.iter().all(|ref current| current.id != last.id)
        });

        self.last_players = current_players.clone();

        Available {
            online_count,
            current_players: current_players.clone().into_iter().map(|ref p| p.name.clone()).collect(),
            joined_players: joined_players.map(|ref p| p.name.clone()).collect(),
            left_players: left_players.map(|ref p| p.name.clone()).collect(),
        }
    }
}
