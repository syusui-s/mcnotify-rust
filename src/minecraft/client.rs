use std::{io, vec, convert};
use std::clone::Clone;
use std::io::Write;
use std::net::{TcpStream, ToSocketAddrs, SocketAddr};
use super::cursor::WritePacketData;
use super::{packet,cursor};
use super::packet::*;

#[derive(Debug)]
pub enum Error {
    ConnectionError(io::Error),
    AddressConvertError(&'static str),
    PacketError(packet::Error)
}

impl convert::From<packet::Error> for Error {
    fn from(err: packet::Error) -> Error {
        Error::PacketError(err)
    }
}

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
            let port_str = iter.next().ok_or(AddressConvertError("invalid port number"))?;
            let hostname = iter.next().ok_or(AddressConvertError("invalid hostname"))?;
            let port : u16 = port_str.parse().map_err(|_| AddressConvertError("invalid port number, parse failed"))?;
            Ok(ServerAddr::new(hostname, port))
        } else {
            Ok(ServerAddr::from_hostname(&self))
        }
    }
}

pub enum State {
    Connected,
    HandShaking,
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

        Ok( Client { server_addr, state: State::Connected, stream: stream } )
    }

    fn write_general_packet(&mut self, packet: &GeneralPacket) -> Result<(), Error> {
        self.stream.write_varint(packet.packet_id.into());
        self.stream.write(packet.body.get_ref());

        Ok(())
    }

    fn write_packet<T: ToGeneralPacket>(&mut self, packet: &T) -> Result<(), Error> {
        self.write_general_packet(&packet.to_general_packet()?)
    }

    pub fn handshake(&mut self) -> Result<(), Error> {
        let packet = HandShakePacket::new(335, "", 25565, NextState::Status);
        self.write_packet(&packet)?;

        Ok(())
    }
}
