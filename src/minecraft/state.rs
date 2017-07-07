use super::packet::PacketType;

#[derive(Debug)]
pub enum Error {
    /// state transition is already done
    AlreadyDone(State),
    NotSatisfy(State),
    InvalidPacketId,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum State {
    Disconnected,
    Connected,
    HandShaking,
    HandShakeDone,
}

impl State {
    pub fn detect_packet_type(&self, id: i32) -> Result<PacketType, Error> {
        use self::Error as E;
        use self::PacketType as PT;

        match id {
            0 => {
                match *self {
                    State::Disconnected  => Err(E::NotSatisfy(State::Connected)),
                    State::Connected     => Err(E::InvalidPacketId),
                    State::HandShaking   => Ok(PT::HandShake),
                    State::HandShakeDone => Ok(PT::List),
                }
            },
            1 => {
                Ok(PT::PingPong)
            },
            _ => Err(E::InvalidPacketId),
        }
    }
}
