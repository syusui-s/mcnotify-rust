use std::{convert, fmt, ops};
use std::collections::HashSet;
use std::iter::FromIterator;
use minecraft::json_data::status::Player as RawPlayer;

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct Player {
    id: String,
    name: String,
}

impl convert::From<RawPlayer> for Player {
    fn from(player: RawPlayer) -> Self {
        Self { id: player.id, name: player.name }
    }
}

impl Player {
    pub fn new(id: &str, name: &str) -> Self {
        Self { id: id.to_owned(), name: name.to_owned() }
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

#[derive(Clone)]
pub struct Players {
    players: HashSet<Player>
}

impl convert::From<Vec<RawPlayer>> for Players {
    fn from(players: Vec<RawPlayer>) -> Self {
        Self::from_iter(players.into_iter())
    }
}

impl convert::From<Vec<Player>> for Players {
    fn from(players: Vec<Player>) -> Self {
        Self::from_iter(players.into_iter())
    }
}

impl FromIterator<RawPlayer> for Players {
    fn from_iter<I: IntoIterator<Item = RawPlayer>>(iter: I) -> Self {
        Self::from_iter(iter.into_iter().map(|raw_player| Player::from(raw_player)))
    }
}

impl FromIterator<Player> for Players {
    fn from_iter<I: IntoIterator<Item = Player>>(iter: I) -> Self {
        Self {
            players: HashSet::from_iter(iter.into_iter())
        }
    }
}

impl<'a, 'b> ops::Sub<&'b Players> for &'a Players {
    type Output = Players;

    fn sub(self, rhs: &'b Players) -> Players {
        let diff = &self.players - &rhs.players;
        Players { players: diff }
    }
}

impl fmt::Display for Players {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut players : Vec<String> = self.players.iter()
            .map(|player| player.name().clone())
            .collect();

        players.sort();

        write!(f, "{}", players.join(", "))
    }
}

impl Players {
    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }
}
