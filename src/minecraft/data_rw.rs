extern crate byteorder;

use std::io::{Read, Write};
use std::{convert, io, string};

use self::byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub const STRING_MAX: usize = 32767;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    MaxStringLenIsTooLong,
    StringConvertError,
    StringHasInvalidLength,
    StringHasInvalidCharacter,
    VarIntIsTooShort,
    VarIntIsTooLong,
    VarLongIsTooShort,
    VarLongIsTooLong,
    StringIsTooLong,
}

impl_convert_for_error!(io::Error, Error::IoError);

impl convert::From<string::FromUtf8Error> for Error {
    fn from(_: string::FromUtf8Error) -> Error {
        Error::StringConvertError
    }
}

pub struct ReadContainer<T> {
    pub content: T,
    pub read_len: usize,
}

impl<T> ReadContainer<T> {
    fn new(content: T, read_len: usize) -> Self {
        Self { content, read_len }
    }
}

type WriteResult = Result<(), Error>;
type ReadResult<T> = Result<ReadContainer<T>, Error>;

pub trait WritePacketData {
    fn write_varint(&mut self, value: i32) -> WriteResult;
    fn write_varlong(&mut self, value: i64) -> WriteResult;
    fn write_byte(&mut self, val: u8) -> WriteResult;
    fn write_unsigned_short(&mut self, val: u16) -> WriteResult;
    fn write_unsigned_int(&mut self, val: u32) -> WriteResult;
    fn write_string(&mut self, string: &str) -> WriteResult;
}

pub trait ReadPacketData {
    fn read_varint(&mut self) -> ReadResult<i32>;
    fn read_varlong(&mut self) -> ReadResult<i64>;
    fn read_byte(&mut self) -> ReadResult<u8>;
    fn read_unsigned_short(&mut self) -> ReadResult<u16>;
    fn read_unsigned_int(&mut self) -> ReadResult<u32>;
    fn read_string_with_max_len(&mut self, max_len: usize) -> ReadResult<String>;
    fn read_string(&mut self) -> ReadResult<String>;
}

macro_rules! write_variable_integer {
    ($name:ident, $signed:ty, $unsigned:ty) => {
        fn $name(&mut self, value: $signed) -> WriteResult {
            let mut uval = value as $unsigned;

            loop {
                let mut tmp = (uval & 0b_0111_1111) as u8;

                uval >>= 7;
                if uval != 0 {
                    tmp |= 0b_1000_0000;
                }

                self.write_all(&[tmp])?;

                if uval == 0 {
                    break;
                }
            }

            Ok(())
        }
    };
}

impl<T> WritePacketData for T
where
    T: Write,
{
    // Writes a VarInt (i32) to the packet body.
    write_variable_integer!(write_varint, i32, u32);

    // Writes a VarLong (i64) to the packet body.
    write_variable_integer!(write_varlong, i64, u64);

    /// Writes an byte (u8) to the packet body.
    fn write_byte(&mut self, val: u8) -> WriteResult {
        self.write_u8(val)?;
        Ok(())
    }

    /// Writes an unsigned short (u16) to the packet body.
    fn write_unsigned_short(&mut self, val: u16) -> WriteResult {
        self.write_u16::<BigEndian>(val)?;
        Ok(())
    }

    /// Writes an unsigned int (u32) to the packet body.
    fn write_unsigned_int(&mut self, val: u32) -> WriteResult {
        self.write_u32::<BigEndian>(val)?;
        Ok(())
    }

    /// Writes a String to the packet body.
    fn write_string(&mut self, string: &str) -> WriteResult {
        // str.len() returns length of BYTES
        let len = string.len();

        if len > 32767 {
            return Err(Error::StringIsTooLong);
        }

        self.write_varint(len as i32)?;

        for chr in string.as_bytes().iter() {
            self.write_byte(*chr)?;
        }

        Ok(())
    }
}

macro_rules! read_variable_integer {
    ($name:ident, $t:ty, $max_len:expr, $err_too_short:path, $err_too_long:path) => {
        fn $name(&mut self) -> ReadResult<$t> {
            let mut result: $t = 0;

            let mut count: usize = 0;
            loop {
                let read = self.read_byte()?.content;
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

            Ok(ReadContainer::new(result, count))
        }
    };
}

impl<T> ReadPacketData for T
where
    T: Read,
{
    read_variable_integer!(
        read_varint,
        i32,
        5_usize,
        Error::VarIntIsTooShort,
        Error::VarIntIsTooLong
    );
    read_variable_integer!(
        read_varlong,
        i64,
        10_usize,
        Error::VarLongIsTooShort,
        Error::VarLongIsTooLong
    );

    fn read_byte(&mut self) -> ReadResult<u8> {
        let result = self.read_u8()?;
        Ok(ReadContainer::new(result, 1))
    }

    fn read_unsigned_short(&mut self) -> ReadResult<u16> {
        let result = self.read_u16::<BigEndian>()?;
        Ok(ReadContainer::new(result, 2))
    }

    fn read_unsigned_int(&mut self) -> ReadResult<u32> {
        let result = self.read_u32::<BigEndian>()?;
        Ok(ReadContainer::new(result, 4))
    }

    fn read_string(&mut self) -> ReadResult<String> {
        self.read_string_with_max_len(STRING_MAX)
    }

    fn read_string_with_max_len(&mut self, max_len: usize) -> ReadResult<String> {
        let len_container = self.read_varint()?;
        let len = len_container.content as usize;

        if max_len > STRING_MAX {
            return Err(Error::MaxStringLenIsTooLong);
        }

        if (len > max_len) || (len > STRING_MAX) {
            return Err(Error::StringIsTooLong);
        } else if len == 0 {
            return Err(Error::StringHasInvalidLength);
        }

        let mut buff = vec![0_u8; len as usize];
        self.read_exact(buff.as_mut_slice())?;

        let result = String::from_utf8(buff)?;

        let read_len = len as usize + len_container.read_len;

        Ok(ReadContainer::new(result, read_len))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Seek, SeekFrom};

    const VARINT_DATA: [(i32, &[u8]); 10] = [
        (0_i32, &[0x00_u8]),
        (1_i32, &[0x01_u8]),
        (2_i32, &[0x02_u8]),
        (19_i32, &[0x13_u8]),
        (127_i32, &[0x7f_u8]),
        (128_i32, &[0x80_u8, 0x01_u8]),
        (255_i32, &[0xff_u8, 0x01_u8]),
        (
            2147483647_i32,
            &[0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0x07_u8],
        ),
        (-1_i32, &[0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0x0f_u8]),
        (
            -2147483648_i32,
            &[0x80_u8, 0x80_u8, 0x80_u8, 0x80_u8, 0x08_u8],
        ),
    ];

    const VARLONG_DATA: [(i64, &[u8]); 11] = [
        (0_i64, &[0x00_u8]),
        (1_i64, &[0x01_u8]),
        (2_i64, &[0x02_u8]),
        (127_i64, &[0x7f_u8]),
        (128_i64, &[0x80_u8, 0x01_u8]),
        (255_i64, &[0xff_u8, 0x01_u8]),
        (
            2147483647_i64,
            &[0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0x07_u8],
        ),
        (
            9223372036854775807_i64,
            &[
                0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0x7f,
            ],
        ),
        (
            -1_i64,
            &[
                0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8,
                0x01,
            ],
        ),
        (
            -2147483648_i64,
            &[
                0x80_u8, 0x80_u8, 0x80_u8, 0x80_u8, 0xf8_u8, 0xff_u8, 0xff_u8, 0xff_u8, 0xff_u8,
                0x01,
            ],
        ),
        (
            -9223372036854775808_i64,
            &[
                0x80_u8, 0x80_u8, 0x80_u8, 0x80_u8, 0x80_u8, 0x80_u8, 0x80_u8, 0x80_u8, 0x80_u8,
                0x01,
            ],
        ),
    ];

    const STRING_DATA: (&str, &[u8]) = (
        "hello world😆",
        &[
            15, 104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 240, 159, 152, 134,
        ],
    );

    macro_rules! test_write_variable_integer {
        ($name:ident, $f:ident, $cases:tt) => {
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
        };
    }

    macro_rules! test_read_variable_integer {
        ($name:ident, $f:ident, $cases:tt) => {
            #[test]
            fn $name() {
                let cases = $cases;
                for &(expect, given) in cases.iter() {
                    let mut cursor = Cursor::new(&given);
                    let result = cursor.$f().unwrap();
                    assert_eq!(result.content, expect);
                    assert_eq!(result.read_len, given.len());
                }
            }
        };
    }

    test_write_variable_integer!(write_varint, write_varint, VARINT_DATA);
    test_write_variable_integer!(write_varlong, write_varlong, VARLONG_DATA);
    test_read_variable_integer!(read_varint, read_varint, VARINT_DATA);
    test_read_variable_integer!(read_varlong, read_varlong, VARLONG_DATA);

    #[test]
    fn write_unsigned_short() {
        let mut cursor = Cursor::new(Vec::new());

        cursor.write_unsigned_short(517_u16).unwrap();
        assert_eq!(cursor.get_ref(), &vec![2_u8, 5_u8]);
    }

    #[test]
    fn write_string() {
        let mut cursor = Cursor::new(Vec::new());

        cursor.write_string("hello world😆").unwrap();
        assert_eq!(
            cursor.get_ref(),
            &vec![15, 104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 240, 159, 152, 134]
        );
    }

    #[test]
    fn read_string() {
        let (expected, given) = STRING_DATA;
        let mut cursor = Cursor::new(given);

        let s = cursor.read_string().unwrap();
        assert_eq!(&s.content, expected);
    }

    #[test]
    fn read_string_with_max_len() {
        let (expected, given) = STRING_DATA;
        let mut cursor = Cursor::new(given);

        let s = cursor.read_string_with_max_len(expected.len()).unwrap();
        assert_eq!(&s.content, expected);
    }
}
