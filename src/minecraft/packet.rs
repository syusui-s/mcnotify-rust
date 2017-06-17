extern crate byteorder;

use std::io::{Cursor, Write};
use self::byteorder::{BigEndian, WriteBytesExt};

#[derive(Debug)]
enum Error {
    InvalidArgument(String),
    FailedToWrite,
}

type WriteResult = Result<(), Error>;

enum PacketType {
    HandShake,
}

struct NoCompressedPacket {
    packet_id: PacketType,
    body: Cursor<Vec<u8>>,
}

macro_rules! write_variable_integer {
    ($name:ident, $signed:ty, $unsigned:ty) => (
        fn $name(&mut self, val: $signed) -> WriteResult {
            let mut uval = val as $unsigned;

            loop {
                let mut tmp = (uval & 0b_0111_1111) as u8;

                uval >>= 7;
                if uval != 0 {
                    tmp |= 0b_1000_0000;
                }

                self.body.write(&[tmp]).or(Err(Error::FailedToWrite))?;

                if uval == 0 { break; }
            }

            Ok(())
        }
    );
}

impl NoCompressedPacket {
    fn new(packet_id: PacketType) -> Self {
        Self {
            packet_id,
            body: Cursor::new(Vec::new())
        }
    }

    /// Writes a VarInt (i32) to the packet body.
    write_variable_integer!(write_varint,  i32, u32);

    /// Writes a VarLong (i64) to the packet body.
    write_variable_integer!(write_varlong, i64, u64);

    /// Writes an unsigned short (u16) to the packet body.
    fn write_unsigned_short(&mut self, val: u16) -> WriteResult {
        self.body.write_u16::<BigEndian>(val)
            .and(Ok(()))
            .or(Err(Error::FailedToWrite))
    }

    /// Writes a String to the packet body.
    fn write_string(&mut self, string: String) -> WriteResult {
        let len = string.len();

        if len > 32767 {
            return Err(Error::InvalidArgument("string length must be <= 32767".to_owned()));
        }

        self.write_varint(len as i32)?;
        for chr in string.encode_utf16() {
            self.write_unsigned_short(chr)?;
        }

        Ok(())
    }
}

struct HandShakeBody {
    protocol_version: i32, // VarInt
    server_address: String,
    server_port: u16,
    next_state: i32, // VarInt
}

impl HandShakeBody {
    fn new(protocol_version: i32, server_address: &str, server_port: u16) -> Self {
        Self {
            protocol_version,
            server_address: server_address.to_owned(),
            server_port,
            next_state: 1
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Seek, SeekFrom};
    use super::{PacketType, NoCompressedPacket};

    macro_rules! test_write_variable_integer {
        ($name:ident, $f:ident, $cases:tt) => (
            #[test]
            fn $name() {
                let mut packet = NoCompressedPacket::new(PacketType::HandShake);

                let cases = $cases;
                for &(given, ref expect) in cases.iter() {
                    packet.body.seek(SeekFrom::Start(0)).unwrap();
                    packet.$f(given).unwrap();
                    assert_eq!(packet.body.get_ref(), expect);
                }
            }
        );
    }

    test_write_variable_integer!(
        test_no_compressed_packet_write_varint,
        write_varint,
        [
        (          0_i32, vec![0x00_u8]),
        (          1_i32, vec![0x01_u8]),
        (          2_i32, vec![0x02_u8]),
        (        127_i32, vec![0x7f_u8]),
        (        128_i32, vec![0x80_u8, 0x01_u8]),
        (        255_i32, vec![0xff_u8, 0x01_u8]),
        ( 2147483647_i32, vec![0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0x07_u8]),
        (         -1_i32, vec![0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0x0f_u8]),
        (-2147483648_i32, vec![0x80_u8, 0x80_u8, 0x80_u8, 0x80_u8, 0x08_u8]),
        ]);

    test_write_variable_integer!(
        test_no_compressed_packet_write_varlong,
        write_varlong,
        [
        (                   0_i64, vec![0x00_u8]),
        (                   1_i64, vec![0x01_u8]),
        (                   2_i64, vec![0x02_u8]),
        (                 127_i64, vec![0x7f_u8]),
        (                 128_i64, vec![0x80_u8, 0x01_u8]),
        (                 255_i64, vec![0xff_u8, 0x01_u8]),
        (          2147483647_i64, vec![0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0x07_u8]),
        ( 9223372036854775807_i64, vec![0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0x7f]),
        (                  -1_i64, vec![0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0x01]),
        (         -2147483648_i64, vec![0x80_u8, 0x80_u8, 0x80_u8, 0x80_u8, 0xf8_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0x01]),
        (-9223372036854775808_i64, vec![0x80_u8, 0x80_u8, 0x80_u8, 0x80_u8, 0x80_u8, 0x80_u8, 0x80_u8, 0x80_u8, 0x80_u8, 0x01]),
        ]);

    #[test]
    fn test_write_unsigned_short() {
        let mut packet = NoCompressedPacket::new(PacketType::HandShake);

        packet.write_unsigned_short(517_u16).unwrap();
        assert_eq!(packet.body.get_ref(), &vec![2_u8, 5_u8]);
    }
}
