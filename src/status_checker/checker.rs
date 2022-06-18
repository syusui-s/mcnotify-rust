use crate::minecraft::{client, packet};
use crate::models::Players;

/// represents a server state.
#[derive(Clone)]
pub enum Status {
    Available {
        online_count: u32,
        current_players: Players,
    },
    Unavailable { reason: String },
}

pub enum StatusDifference {
    /// some player joined into or left from server
    PlayerChange {
        online_count: u32,
        current_players: Players,
        joined_players: Players,
        left_players: Players,
    },
    /// recovered from
    Recover {
        online_count: u32,
        current_players: Players,
    },
    Down { reason: String },
    None { latest_status: Status },
}

impl StatusDifference {
    fn from_between(latest_status: &Status, current_status: &Status) -> Self {
        use self::Status::*;
        use self::StatusDifference::*;

        match (latest_status, current_status) {
            (&Unavailable { .. },
             &Unavailable { .. }) => {
                None { latest_status: current_status.clone() }
            }
            (&Unavailable { .. },
             &Available { online_count, ref current_players, ..  }) => {
                Recover {
                    online_count,
                    current_players: current_players.clone(),
                }
            }
            (&Available { .. },
             &Unavailable { ref reason }) => {
                Down { reason: reason.clone() }
            },
            (&Available { current_players: ref latest_players, .. },
             &Available { online_count, ref current_players, }) => {
                let joined_players = current_players - latest_players;
                let left_players = latest_players - current_players;

                if !joined_players.is_empty() || !left_players.is_empty() {
                    PlayerChange {
                        online_count,
                        current_players: current_players.clone(),
                        joined_players,
                        left_players,
                    }
                } else {
                    None { latest_status: latest_status.clone() }
                }
            }
        }
    }
}

pub struct StatusChecker {
    hostname: String,
    port: u16,
    latest_status: Status,
}

impl StatusChecker {
    pub fn new(hostname: &str, port: u16) -> Self {
        Self {
            hostname: hostname.to_owned(),
            port,
            latest_status: Status::Unavailable { reason: "on start".to_owned() },
        }
    }

    pub fn get_status_difference(&mut self) -> StatusDifference {
        let current_status = self.get_status();
        let difference = StatusDifference::from_between(&self.latest_status, &current_status);
        self.latest_status = current_status;

        difference
    }

    fn get_status(&mut self) -> Status {
        use self::Status::*;

        let address = client::ServerAddr::new(&self.hostname, self.port);

        // get status
        let mut cli = match client::Client::connect(address) {
            Ok(cli) => cli,
            Err(_) => {
                return Unavailable {
                    reason: format!("Couldn't connect to {}:{}", self.hostname, self.port),
                }
            }
        };

        match cli.handshake(packet::NextState::Status) {
            Ok(_) => {}
            Err(_) => {
                return Unavailable {
                    reason: format!("Couldn't handshake with {}:{}", self.hostname, self.port),
                }
            }
        }

        let status = match cli.list() {
            Ok(res) => res,
            Err(e) => {
                return Status::Unavailable { reason: format!("List Request was failed : {:?}", e) };
            }
        };

        // build information
        let online_count = status.players.online;
        let current_players = Players::from(status.players.sample.unwrap_or_else(|| vec![]));

        Status::Available {
            online_count,
            current_players,
        }
    }
}
