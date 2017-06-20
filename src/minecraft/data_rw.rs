extern crate byteorder;

use std::{io,convert,string};
use std::io::{Read, Write};

use self::byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    StringConvertError,
    VarIntIsTooShort,
    VarIntIsTooLong,
    VarLongIsTooShort,
    VarLongIsTooLong,
    StringIsTooLong,
}

impl_convert_for_error!(io::Error, Error::IoError);

impl convert::From<string::FromUtf16Error> for Error {
    fn from(err: string::FromUtf16Error) -> Error {
        Error::StringConvertError
    }
}

type WriteResult = Result<(), Error>;

pub trait WritePacketData {
    fn write_varint(&mut self, i32) -> WriteResult;
    fn write_varlong(&mut self, i64) -> WriteResult;
    fn write_unsigned_short(&mut self, val: u16) -> WriteResult;
    fn write_string(&mut self, string: &str) -> WriteResult;
}

pub trait ReadPacketData {
    fn read_varint(&mut self) -> Result<i32, Error>;
    fn read_varlong(&mut self) -> Result<i64, Error>;
    fn read_unsigned_short(&mut self) -> Result<u16, Error>;
    fn read_string(&mut self) -> Result<String, Error>;
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
            return Err(Error::StringIsTooLong);
        }

        self.write_varint(len as i32)?;
        for chr in string.encode_utf16() {
            self.write_unsigned_short(chr)?;
        }

        Ok(())
    }
}

macro_rules! read_variable_integer {
    ($name:ident, $t:ty, $max_len:expr, $err_too_short:path, $err_too_long:path) => (
        fn $name(&mut self) -> Result<$t, Error> {
            let mut bytes = self.bytes();
            let mut result : $t = 0;

            let mut count = 0;
            loop {
                let read = bytes.next().ok_or($err_too_short)??;
                let value = (read & 0b_0111_1111) as $t;
                result |= value << (7 * count);

                count += 1;
                if count > $max_len {
                    return Err($err_too_long);
                }

                if (read & 0b_1000_0000) == 0 {
                    break;
                }
            }

            Ok(result)
        }
    );
}

impl<T> ReadPacketData for T where T: Read {
    read_variable_integer!(read_varint,  i32,  5_i32, Error::VarIntIsTooShort,  Error::VarIntIsTooLong);
    read_variable_integer!(read_varlong, i64, 10_i64, Error::VarLongIsTooShort, Error::VarLongIsTooLong);

    fn read_unsigned_short(&mut self) -> Result<u16, Error> {
        Ok(self.read_u16::<BigEndian>()?)
    }

    fn read_string(&mut self) -> Result<String, Error> {
        let len = self.read_varint()?;

        if len > 32767 {
            return Err(Error::StringIsTooLong);
        } else if len == 0 {
            return Ok("".to_owned());
        }

        let mut vec = Vec::<u16>::with_capacity(len as usize);
        for i in 0..len {
            vec.push(self.read_unsigned_short()?);
        }

        Ok(String::from_utf16(vec.as_ref())?)
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

    #[test]
    fn read_string() {
        let vec = vec![11, 0, 104, 0, 101, 0, 108, 0, 108, 0, 111, 0, 32, 0, 119, 0, 111, 0, 114, 0, 108, 0, 100];
        let mut cursor = Cursor::new(vec);

        let s = cursor.read_string().unwrap();
        assert_eq!(s, "hello world".to_owned());
    }
}
