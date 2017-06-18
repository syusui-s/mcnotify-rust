use std::{io, convert};
use std::ops::Deref;
use super::{data_rw,json_data};
use super::data_rw::WritePacketData;

#[derive(Debug)]
pub enum Error {
    InvalidArgument(String),
    DataRWError(data_rw::Error),
}

impl_convert_for_error!(data_rw::Error, Error::DataRWError);

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PacketType {
    HandShake,
    List,
    PingPong,
}

impl Into<i32> for PacketType {
    fn into(self) -> i32 {
        use self::PacketType::*;

        match self {
            HandShake => 0,
            List => 0,
            PingPong => 1,
        }
    }
}

#[derive(Debug)]
pub enum NextState {
    Status,
    Login,
}

impl Into<i32> for NextState {
    fn into(self) -> i32 {
        use self::NextState::*;

        match self {
            Status => 1,
            Login => 2,
        }
    }
}
#[derive(Debug)]
pub struct GeneralPacket {
    pub packet_id: PacketType,
    pub body: io::Cursor<Vec<u8>>,
}

impl GeneralPacket {
    pub fn new(packet_id: PacketType) -> Self {
        Self {
            packet_id,
            body: io::Cursor::new(Vec::new())
        }
    }
}

pub trait ToGeneralPacket {
    fn to_general_packet(&self) -> Result<GeneralPacket, Error>;
}

#[derive(Debug)]
pub struct HandShakePacket {
    protocol_version: i32, // VarInt
    server_address: String,
    server_port: u16,
    next_state: i32, // VarInt
}

impl HandShakePacket {
    pub fn new(protocol_version: i32, server_address: &str, server_port: u16, next_state: NextState) -> Self {
        Self {
            protocol_version,
            server_address: server_address.to_owned(),
            server_port,
            next_state: next_state.into()
        }
    }
}

impl ToGeneralPacket for HandShakePacket {
    fn to_general_packet(&self) -> Result<GeneralPacket, Error> {
        let mut packet = GeneralPacket::new(PacketType::HandShake);

        packet.body.write_varint(self.protocol_version)?;
        packet.body.write_string(self.server_address.deref())?;
        packet.body.write_unsigned_short(self.server_port)?;
        packet.body.write_varint(self.next_state)?;

        Ok(packet)
    }
}

pub struct ListRequestPacket;

impl ListRequestPacket {
    pub fn new() -> Self {
        Self{}
    }
}

impl ToGeneralPacket for ListRequestPacket {
    fn to_general_packet(&self) -> Result<GeneralPacket, Error> {
        let mut packet = GeneralPacket::new(PacketType::List);
        Ok(packet)
    }
}

pub struct ListResponsePacket {
    status: json_data::status::Status,
}

/*
impl convert::From<GeneralPacket> for ListResponsePacket {
    fn from(packet: GeneralPacket) -> ListResponsePacket {
    }
}
*/

#[cfg(test)]
mod tests {
    use std::io::Write;
    use super::*;

    #[test]
    fn handshake_to_general() {
        let from = HandShakePacket::new(335, "localhost", 25565, NextState::Status);
        let mut to = GeneralPacket::new(PacketType::HandShake);
        to.body.write(&[207, 2, 9, 0, 108, 0, 111, 0, 99, 0, 97, 0, 108, 0, 104, 0, 111, 0, 115, 0, 116, 99, 221, 1]).unwrap();

        let converted = from.to_general_packet().unwrap();

        assert_eq!(converted.packet_id, to.packet_id);
        assert_eq!(converted.body.get_ref(), to.body.get_ref());
    }
}
