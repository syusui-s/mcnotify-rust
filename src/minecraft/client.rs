use super::packet::*;
use super::packet_rw::{ReadPacket, WritePacket};
use super::state::State;
use super::{data_rw, json_data, packet, packet_rw, state};
use std::net::{Shutdown, SocketAddr, TcpStream, ToSocketAddrs};
use std::{convert, io, vec};

#[derive(Debug)]
pub enum Error {
    ConnectionError(io::Error),
    AddressConvertError(String),
    DataRWError(data_rw::Error),
    PacketError(packet::Error),
    PacketRWError(packet_rw::Error),
    StateError(state::Error),
    IoError(io::Error),
    InvalidPacketId,
}

impl_convert_for_error!(data_rw::Error, Error::DataRWError);
impl_convert_for_error!(io::Error, Error::IoError);
impl_convert_for_error!(packet::Error, Error::PacketError);
impl_convert_for_error!(packet_rw::Error, Error::PacketRWError);
impl_convert_for_error!(state::Error, Error::StateError);

/// Server address consists of the pair of hostname and port number
#[derive(Clone)]
pub struct ServerAddr {
    hostname: String,
    port: u16,
}

impl ServerAddr {
    pub fn new(hostname: &str, port: u16) -> Self {
        Self {
            hostname: hostname.into(),
            port,
        }
    }

    /// a port number will be set to default value, 25565.
    pub fn from_hostname(hostname: &str) -> Self {
        Self {
            hostname: hostname.into(),
            port: 25565,
        }
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

impl ToServerAddr for ServerAddr {
    fn to_server_addr(&self) -> Result<ServerAddr, Error> {
        Ok(self.clone())
    }
}

impl<'a> ToServerAddr for &'a str {
    fn to_server_addr(&self) -> Result<ServerAddr, Error> {
        use self::Error::AddressConvertError;

        if self.contains(':') {
            let mut iter = self.rsplitn(2, ':');
            let port_str = iter
                .next()
                .ok_or_else(|| AddressConvertError("invalid port number".to_owned()))?;
            let hostname = iter
                .next()
                .ok_or_else(|| AddressConvertError("invalid hostname".to_owned()))?;
            let port: u16 = port_str
                .parse()
                .map_err(|_| AddressConvertError("invalid port number, parse failed".to_owned()))?;
            Ok(ServerAddr::new(hostname, port))
        } else {
            Ok(ServerAddr::from_hostname(self))
        }
    }
}

pub struct Client {
    server_addr: ServerAddr,
    state: State,
    stream: TcpStream,
}

impl Client {
    pub fn connect<A: ToServerAddr>(addr: A) -> Result<Self, Error> {
        let server_addr = addr.to_server_addr()?;
        let stream = TcpStream::connect(&server_addr).map_err(Error::ConnectionError)?;

        stream.set_read_timeout(None)?;

        Ok(Client {
            server_addr,
            state: State::HandShaking,
            stream,
        })
    }

    pub fn handshake(&mut self, next_state: NextState) -> Result<(), Error> {
        const SUPPORTED_VERSION: i32 = 335;

        if self.state != State::HandShaking {
            return Err(Error::from(state::Error::AlreadyDone(State::HandShaking)));
        }

        let packet = HandShakePacket::new(
            SUPPORTED_VERSION,
            &self.server_addr.hostname,
            self.server_addr.port,
            next_state,
        );
        self.stream.write_packet(&packet)?;

        self.state = State::HandShakeDone;

        Ok(())
    }

    pub fn list(&mut self) -> Result<json_data::status::Status, Error> {
        if self.state == State::HandShaking {
            self.handshake(NextState::Status)?;
        }

        let packet = ListRequestPacket::new();
        self.stream.write_packet(&packet)?;

        let packet = self.stream.read_packet::<ListResponsePacket>(self.state)?;
        let status = packet.status;

        Ok(status)
    }

    pub fn shutdown(&mut self) {
        self.stream.shutdown(Shutdown::Both).unwrap();
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.shutdown();
    }
}
