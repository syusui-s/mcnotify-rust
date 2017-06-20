use std::{io, vec, convert};
use std::io::{Write, Cursor};
use std::net::{TcpStream, ToSocketAddrs, SocketAddr};
use super::data_rw::WritePacketData;
use super::{packet,data_rw};
use super::packet::*;

#[derive(Debug)]
pub enum StateError {
    /// state transition is already done
    AlreadyDone(State),
    NotSatisfy(State),
}

#[derive(Debug)]
pub enum Error {
    ConnectionError(io::Error),
    AddressConvertError(String),
    StateError(StateError),
    PacketError(packet::Error),
    DataRWError(data_rw::Error),
    IoError(io::Error),
}

impl_convert_for_error!(data_rw::Error, Error::DataRWError);
impl_convert_for_error!(io::Error, Error::IoError);
impl_convert_for_error!(packet::Error, Error::PacketError);
impl_convert_for_error!(StateError, Error::StateError);

/// Server address consists of the pair of hostname and port number
#[derive(Clone)]
pub struct ServerAddr {
    hostname: String,
    port: u16,
}

impl ServerAddr {
    pub fn new(hostname: &str, port: u16) -> Self {
        Self { hostname: hostname.into(), port }
    }

    /// a port number will be set to default value, 25565.
    pub fn from_hostname(hostname: &str) -> Self {
        Self { hostname: hostname.into(), port: 25565 }
    }
}

impl ToSocketAddrs for ServerAddr {
    type Iter = vec::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        format!("{}:{}", self.hostname, self.port).to_socket_addrs()
    }
}

pub trait ToServerAddr {
    fn to_server_addr(&self) -> Result<ServerAddr, Error>;
}

/*
impl ToServerAddr for ServerAddr {
    fn to_server_addr(&self) -> Result<ServerAddr, Error> {
        Ok(self)
    }
}
*/

impl<'a> ToServerAddr for &'a str {
    fn to_server_addr(&self) -> Result<ServerAddr, Error> {
        use self::Error::AddressConvertError;

        if self.contains(":") {
            let mut iter = self.rsplitn(2, ':');
            let port_str = iter.next().ok_or(AddressConvertError("invalid port number".to_owned()))?;
            let hostname = iter.next().ok_or(AddressConvertError("invalid hostname".to_owned()))?;
            let port : u16 = port_str.parse().map_err(|_| AddressConvertError("invalid port number, parse failed".to_owned()))?;
            Ok(ServerAddr::new(hostname, port))
        } else {
            Ok(ServerAddr::from_hostname(&self))
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum State {
    HandShaking,
    HandShakeDone,
}

pub struct Client {
    server_addr: ServerAddr,
    state: State,
    stream: TcpStream,
}

impl Client {
    pub fn connect<A : ToServerAddr>(addr: A) -> Result<Self, Error> {
        let server_addr = addr.to_server_addr()?;
        let stream = TcpStream::connect(&server_addr)
            .map_err(|err| Error::ConnectionError(err))?;

        Ok( Client { server_addr, state: State::HandShaking, stream: stream } )
    }

    fn write_general_packet(&mut self, packet: &GeneralPacket) -> Result<(), Error> {
        let mut packet_id_buff = Cursor::new(Vec::with_capacity(5));
        packet_id_buff.write_varint(packet.packet_id.into())?;

        let packet_id = packet_id_buff.get_ref();
        let body = packet.body.get_ref();
        let len = (body.len() + packet_id.len()) as i32;

        self.stream.write_varint(len);
        self.stream.write(packet_id)?;
        self.stream.write(body)?;
        Ok(())
    }

    fn write_packet<T: ToGeneralPacket>(&mut self, packet: &T) -> Result<(), Error> {
        self.write_general_packet(&packet.to_general_packet()?)
    }

    fn read_general_packet(&mut self) {
    }

    pub fn handshake(&mut self) -> Result<(), Error> {
        if self.state != State::HandShaking {
            return Err(Error::from(StateError::AlreadyDone(State::HandShaking)));
        }

        let packet = HandShakePacket::new(335, &self.server_addr.hostname, self.server_addr.port, NextState::Status);
        self.write_packet(&packet)?;

        self.state = State::HandShakeDone;

        Ok(())
    }

    pub fn list(&mut self) -> Result<(), Error> {
        if self.state == State::HandShaking {
            self.handshake();
        }

        let packet = ListRequestPacket::new();
        self.write_packet(&packet)?;

        Ok(())
    }
}
