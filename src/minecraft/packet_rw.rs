use super::data_rw::{ReadPacketData, WritePacketData};
use super::packet::*;
use super::state::State;
use super::{data_rw, packet, state};
use std::io::{Cursor, Read, Write};
use std::{convert, io};

#[derive(Debug)]
pub enum Error {
    DataRWError(data_rw::Error),
    IOError(io::Error),
    PacketError(packet::Error),
    StateError(state::Error),
    PacketHasNegativeLength,
}

impl_convert_for_error!(data_rw::Error, Error::DataRWError);
impl_convert_for_error!(io::Error, Error::IOError);
impl_convert_for_error!(packet::Error, Error::PacketError);
impl_convert_for_error!(state::Error, Error::StateError);

pub trait WritePacket {
    fn write_general_packet(&mut self, packet: &GeneralPacket) -> Result<(), Error>;
    fn write_packet<P>(&mut self, packet: &P) -> Result<(), Error>
    where
        P: ToGeneralPacket;
}

pub trait ReadPacket {
    fn read_general_packet(&mut self, state: State) -> Result<GeneralPacket, Error>;
    fn read_packet<P>(&mut self, state: State) -> Result<P, Error>
    where
        P: FromGeneralPacket;
}

impl<T> WritePacket for T
where
    T: Write,
{
    fn write_general_packet(&mut self, packet: &GeneralPacket) -> Result<(), Error> {
        let mut packet_id_buff = Cursor::new(Vec::with_capacity(5));
        packet_id_buff.write_varint(packet.packet_id.into())?;

        let packet_id = packet_id_buff.get_ref();
        let body = packet.body.get_ref();
        let len = (body.len() + packet_id.len()) as i32;

        self.write_varint(len)?;
        self.write(packet_id)?;
        self.write(body)?;

        Ok(())
    }

    fn write_packet<P>(&mut self, packet: &P) -> Result<(), Error>
    where
        P: ToGeneralPacket,
    {
        self.write_general_packet(&packet.to_general_packet()?)?;

        Ok(())
    }
}

impl<T> ReadPacket for T
where
    T: Read,
{
    fn read_general_packet(&mut self, state: State) -> Result<GeneralPacket, Error> {
        // Length
        let len_container = self.read_varint()?;
        let len = len_container.content;

        if len < 0 {
            return Err(Error::PacketHasNegativeLength);
        }

        // Packet ID
        let packet_id_container = self.read_varint()?;
        let packet_id = packet_id_container.content;

        // Body
        let body_len = (len as usize) - packet_id_container.read_len;
        let mut body = vec![0_u8; body_len];
        self.read_exact(body.as_mut_slice())?;

        // Construct
        let packet = GeneralPacket::with_body_vec(state.detect_packet_type(packet_id)?, body);

        Ok(packet)
    }

    fn read_packet<P>(&mut self, state: State) -> Result<P, Error>
    where
        P: FromGeneralPacket,
    {
        let mut general_packet = self.read_general_packet(state)?;
        P::from_general_packet(&mut general_packet).map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {}
