/*
enum Error {
    ConnectionError(String)
}

enum State {
    Constructed,
    Connected,
    HandShaking,
}

struct Client {
    state: State,
    stream: TcpStream,
}

struct Status {
    raw: String,
}

impl Client {
    fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self, Error> {
        ToSocketAddrs

            TcpStream::connect(addr)
            .map(|stream| Client { state: State::Connected, stream: stream } )
            .map_err(|err| Error::ConnectionError(format!("{:?}", err.kind())) )
    }

    fn handshake(&mut self) {
    }

    fn ping(&mut self) -> Result<Status, Error> {
        Ok(Status { raw: "".to_owned() })
    }
}
*/
