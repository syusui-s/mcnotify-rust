use crate::minecraft::json_data::status::Player as RawPlayer;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::{convert, fmt, ops};

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct Player {
    id: String,
    name: String,
}

impl convert::From<RawPlayer> for Player {
    fn from(player: RawPlayer) -> Self {
        Self {
            id: player.id,
            name: player.name,
        }
    }
}

impl Player {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_owned(),
            name: name.to_owned(),
        }
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct Players {
    players: HashSet<Player>,
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
        Self::from_iter(iter.into_iter().map(Player::from))
    }
}

impl FromIterator<Player> for Players {
    fn from_iter<I: IntoIterator<Item = Player>>(iter: I) -> Self {
        Self {
            players: HashSet::from_iter(iter.into_iter()),
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
        let mut players: Vec<String> = self
            .players
            .iter()
            .map(|player| player.name().clone())
            .collect();

        players.sort();

        write!(f, "{}", players.join(", "))
    }
}

impl fmt::Debug for Players {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Players {
    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_players_format() {
        let players = Players::from(vec![
            Player::new("idA", "A"),
            Player::new("idB", "B"),
            Player::new("idC", "C"),
        ]);
        assert_eq!(format!("{}", players), "A, B, C");
    }

    #[test]
    fn test_players_sub() {
        let players_lhs = Players::from(vec![
            Player::new("idA", "A"),
            Player::new("idB", "B"),
            Player::new("idC", "C"),
        ]);
        let players_rhs = Players::from(vec![Player::new("idC", "C")]);

        let actual = &players_lhs - &players_rhs;
        let expected = Players::from(vec![Player::new("idA", "A"), Player::new("idB", "B")]);

        assert_eq!(actual, expected);
    }
}
