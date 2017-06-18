extern crate byteorder;

use std::{io,convert};
use std::io::{Write};

use self::byteorder::{BigEndian, WriteBytesExt};

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    ArgumentError(&'static str),
}

impl convert::From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

type WriteResult = Result<(), Error>;

pub trait WritePacketData {
    fn write_varint(&mut self, i32) -> WriteResult;
    fn write_varlong(&mut self, i64) -> WriteResult;
    fn write_unsigned_short(&mut self, val: u16) -> WriteResult;
    fn write_string(&mut self, string: &str) -> WriteResult;
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

                self.write(&[tmp])?;

                if uval == 0 { break; }
            }

            Ok(())
        }
    );
}

impl<T> WritePacketData for T where T: Write {
    /// Writes a VarInt (i32) to the packet body.
    write_variable_integer!(write_varint,  i32, u32);

    /// Writes a VarLong (i64) to the packet body.
    write_variable_integer!(write_varlong, i64, u64);

    /// Writes an unsigned short (u16) to the packet body.
    fn write_unsigned_short(&mut self, val: u16) -> WriteResult {
        self.write_u16::<BigEndian>(val)?;
        Ok(())
    }

    /// Writes a String to the packet body.
    fn write_string(&mut self, string: &str) -> WriteResult {
        let len = string.len();

        if len > 32767 {
            return Err(Error::ArgumentError("string length must be <= 32767"));
        }

        self.write_varint(len as i32)?;
        for chr in string.encode_utf16() {
            self.write_unsigned_short(chr)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Seek, SeekFrom, Cursor};
    use super::*;

    macro_rules! test_write_variable_integer {
        ($name:ident, $f:ident, $cases:tt) => (
            #[test]
            fn $name() {
                let mut cursor = Cursor::new(Vec::new());

                let cases = $cases;
                for &(given, ref expect) in cases.iter() {
                    cursor.seek(SeekFrom::Start(0)).unwrap();
                    cursor.$f(given).unwrap();
                    assert_eq!(cursor.get_ref(), expect);
                }
            }
            );
    }

    test_write_variable_integer!(
        write_varint,
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
        write_varlong,
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
    fn write_unsigned_short() {
        let mut cursor = Cursor::new(Vec::new());

        cursor.write_unsigned_short(517_u16).unwrap();
        assert_eq!(cursor.get_ref(), &vec![2_u8, 5_u8]);
    }

    #[test]
    fn write_string() {
        let mut cursor = Cursor::new(Vec::new());

        cursor.write_string("hello world").unwrap();
        assert_eq!(
            cursor.get_ref(),
            &vec![11, 0, 104, 0, 101, 0, 108, 0, 108, 0, 111, 0, 32, 0, 119, 0, 111, 0, 114, 0, 108, 0, 100]);
    }
}
