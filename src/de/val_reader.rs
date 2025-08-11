use std::io::Read;
use std::marker::PhantomData;

use byteorder::ByteOrder;
use byteorder::ReadBytesExt;

pub struct BinValReader<E: ByteOrder> {
    _endian: PhantomData<E>,
}

pub struct AsciiValReader {}

pub(crate) trait ScalarReader {
    fn read_i8(reader: impl Read) -> Result<i8, std::io::Error>;
    fn read_u8(reader: impl Read) -> Result<u8, std::io::Error>;
    fn read_i16(reader: impl Read) -> Result<i16, std::io::Error>;
    fn read_u16(reader: impl Read) -> Result<u16, std::io::Error>;
    fn read_i32(reader: impl Read) -> Result<i32, std::io::Error>;
    fn read_u32(reader: impl Read) -> Result<u32, std::io::Error>;
    fn read_f32(reader: impl Read) -> Result<f32, std::io::Error>;
    fn read_f64(reader: impl Read) -> Result<f64, std::io::Error>;
}

impl<E: ByteOrder> ScalarReader for BinValReader<E> {
    fn read_i8(mut reader: impl Read) -> Result<i8, std::io::Error> {
        reader.read_i8()
    }

    fn read_u8(mut reader: impl Read) -> Result<u8, std::io::Error> {
        reader.read_u8()
    }

    fn read_i16(mut reader: impl Read) -> Result<i16, std::io::Error> {
        reader.read_i16::<E>()
    }

    fn read_u16(mut reader: impl Read) -> Result<u16, std::io::Error> {
        reader.read_u16::<E>()
    }

    fn read_i32(mut reader: impl Read) -> Result<i32, std::io::Error> {
        reader.read_i32::<E>()
    }

    fn read_u32(mut reader: impl Read) -> Result<u32, std::io::Error> {
        reader.read_u32::<E>()
    }

    fn read_f32(mut reader: impl Read) -> Result<f32, std::io::Error> {
        reader.read_f32::<E>()
    }

    fn read_f64(mut reader: impl Read) -> Result<f64, std::io::Error> {
        reader.read_f64::<E>()
    }
}

impl ScalarReader for AsciiValReader {
    fn read_i8(reader: impl Read) -> Result<i8, std::io::Error> {
        Self::read_ascii_token(reader)?.parse::<i8>().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to parse i8 from ASCII",
            )
        })
    }

    fn read_u8(reader: impl Read) -> Result<u8, std::io::Error> {
        Self::read_ascii_token(reader)?.parse::<u8>().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to parse u8 from ASCII",
            )
        })
    }

    fn read_i16(reader: impl Read) -> Result<i16, std::io::Error> {
        Self::read_ascii_token(reader)?.parse::<i16>().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to parse i16 from ASCII",
            )
        })
    }

    fn read_u16(reader: impl Read) -> Result<u16, std::io::Error> {
        Self::read_ascii_token(reader)?.parse::<u16>().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to parse u16 from ASCII",
            )
        })
    }

    fn read_i32(reader: impl Read) -> Result<i32, std::io::Error> {
        Self::read_ascii_token(reader)?.parse::<i32>().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to parse i32 from ASCII",
            )
        })
    }

    fn read_u32(reader: impl Read) -> Result<u32, std::io::Error> {
        Self::read_ascii_token(reader)?.parse::<u32>().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to parse u32 from ASCII",
            )
        })
    }

    fn read_f32(reader: impl Read) -> Result<f32, std::io::Error> {
        Self::read_ascii_token(reader)?.parse::<f32>().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to parse f32 from ASCII",
            )
        })
    }

    fn read_f64(reader: impl Read) -> Result<f64, std::io::Error> {
        Self::read_ascii_token(reader)?.parse::<f64>().map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to parse f64 from ASCII",
            )
        })
    }
}

impl AsciiValReader {
    fn read_ascii_token(mut reader: impl Read) -> Result<String, std::io::Error> {
        let mut token = String::new();

        loop {
            let mut byte = [0u8; 1];
            match reader.read_exact(&mut byte) {
                Ok(_) => {
                    let ch = byte[0] as char;
                    if ch.is_ascii_whitespace() {
                        if !token.is_empty() {
                            break;
                        }
                    } else {
                        token.push(ch);
                    }
                }
                Err(e) => return Err(e),
            }
        }

        if token.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "No valid ASCII token found",
            ));
        }

        Ok(token)
    }
}
