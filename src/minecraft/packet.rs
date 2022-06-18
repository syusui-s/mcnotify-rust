extern crate serde;
extern crate serde_json;

use super::data_rw::{ReadPacketData, WritePacketData};
use super::{data_rw, json_data};
use std::ops::Deref;
use std::{convert, io};

/// * defines packet structure
/// * defines methods for each packet
/// * defines some enums for packets
/// * general packet to specific packet conversion
/// * specific packet to general packet conversion

#[derive(Debug)]
pub enum Error {
    DataRWError(data_rw::Error),
    JsonError(serde_json::Error),
}

impl_convert_for_error!(data_rw::Error, Error::DataRWError);
impl_convert_for_error!(serde_json::Error, Error::JsonError);

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PacketType {
    HandShake,
    List,
    PingPong,
}

impl convert::Into<i32> for PacketType {
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

impl convert::Into<i32> for NextState {
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
            body: io::Cursor::new(Vec::new()),
        }
    }

    pub fn with_body_vec(packet_id: PacketType, body_vec: Vec<u8>) -> Self {
        Self {
            packet_id,
            body: io::Cursor::new(body_vec),
        }
    }
}

pub trait ToGeneralPacket {
    fn to_general_packet(&self) -> Result<GeneralPacket, Error>;
}

pub trait FromGeneralPacket
where
    Self: Sized,
{
    fn from_general_packet(general_packet: &mut GeneralPacket) -> Result<Self, Error>;
}

#[derive(Debug)]
pub struct HandShakePacket {
    protocol_version: i32, // VarInt
    server_address: String,
    server_port: u16,
    next_state: i32, // VarInt
}

impl HandShakePacket {
    pub fn new(
        protocol_version: i32,
        server_address: &str,
        server_port: u16,
        next_state: NextState,
    ) -> Self {
        Self {
            protocol_version,
            server_address: server_address.to_owned(),
            server_port,
            next_state: next_state.into(),
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
        Self {}
    }
}

impl ToGeneralPacket for ListRequestPacket {
    fn to_general_packet(&self) -> Result<GeneralPacket, Error> {
        let packet = GeneralPacket::new(PacketType::List);
        Ok(packet)
    }
}

pub struct ListResponsePacket {
    pub status: json_data::status::Status,
}

impl ListResponsePacket {
    pub fn new(status: json_data::status::Status) -> Self {
        Self { status }
    }

    pub fn status(&self) -> &json_data::status::Status {
        &self.status
    }
}

impl FromGeneralPacket for ListResponsePacket {
    fn from_general_packet(general_packet: &mut GeneralPacket) -> Result<Self, Error> {
        let body_string = general_packet.body.read_string()?.content;
        let status: json_data::status::Status = serde_json::from_str(&body_string)?;
        Ok(ListResponsePacket::new(status))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handshake_to_general() {
        let from = HandShakePacket::new(335, "localhost", 25565, NextState::Status);
        let to = GeneralPacket::with_body_vec(
            PacketType::HandShake,
            vec![
                207, 2, 9, 108, 111, 99, 97, 108, 104, 111, 115, 116, 99, 221, 1,
            ],
        );

        let converted = from.to_general_packet().unwrap();

        assert_eq!(converted.packet_id, to.packet_id);
        assert_eq!(converted.body.get_ref(), to.body.get_ref());
    }

    #[test]
    fn list_response_from_general() {
        use self::json_data::chat::*;
        use self::json_data::status::*;
        use std::str::FromStr;

        let mut from = GeneralPacket::with_body_vec(PacketType::List, {
            let mut header: Vec<u8> = vec![0x8c, 0x0b];
            let mut body: Vec<u8> = Vec::from(
                r#"{"description":"A Minecraft Server","players":{"max":20,"online":0},"version":{"name":"1.7.10","protocol":5},"modinfo":{"type":"FML","modList":[{"modid":"mcp","version":"9.05"},{"modid":"FML","version":"7.10.99.99"},{"modid":"Forge","version":"10.13.4.1614"},{"modid":"clayiumtransformer","version":"0.4.1"},{"modid":"CodeChickenCore","version":"1.0.7.47"},{"modid":"NotEnoughItems","version":"1.0.5.120"},{"modid":"\u003cCoFH ASM\u003e","version":"000"},{"modid":"net.minecraft.scalar.cutall.mod_CutAllSMP","version":"2.5.0"},{"modid":"CoFHCore","version":"1.7.10R3.1.4"},{"modid":"BuildCraft|Core","version":"7.1.22"},{"modid":"BuildCraft|Transport","version":"7.1.22"},{"modid":"BuildCraft|Factory","version":"7.1.22"},{"modid":"BuildCraft|Silicon","version":"7.1.22"},{"modid":"BuildCraft|Robotics","version":"7.1.22"},{"modid":"BuildCraft|Energy","version":"7.1.22"},{"modid":"BuildCraft|Builders","version":"7.1.22"},{"modid":"ChickenChunks","version":"1.3.4.16"},{"modid":"IC2","version":"2.2.828-experimental"},{"modid":"ThermalFoundation","version":"1.7.10R1.2.6"},{"modid":"ThermalExpansion","version":"1.7.10R4.1.5"},{"modid":"clayium","version":"0.4.6.36.hotfix2"},{"modid":"exnihilo","version":"1.38-53"},{"modid":"exastris","version":"MC1.7.10-1.16-36"},{"modid":"inventorytweaks","version":"1.59-dev-152-cf6e263"},{"modid":"JABBA","version":"1.2.2"},{"modid":"ThermalDynamics","version":"1.7.10R1.2.1"}]}}"#,
            );

            header.append(&mut body);

            header
        });

        let to = ListResponsePacket::new(Status {
            version: Version {
                name: "1.7.10".to_owned(),
                protocol: 5u32,
            },
            description: Chat::from_str("A Minecraft Server").unwrap(),
            players: Players {
                max: 20,
                online: 0,
                sample: None,
            },
        });

        let converted = ListResponsePacket::from_general_packet(&mut from).unwrap();

        assert_eq!(converted.status.version.name, to.status.version.name);
    }
}
