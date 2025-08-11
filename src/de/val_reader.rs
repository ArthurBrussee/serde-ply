use std::io::Read;
use std::marker::PhantomData;

use byteorder::ByteOrder;
use byteorder::ReadBytesExt;
use thiserror::Error;

pub struct BinValReader<E: ByteOrder> {
    _endian: PhantomData<E>,
}

impl<E: ByteOrder> BinValReader<E> {
    pub(crate) fn new() -> Self {
        Self {
            _endian: PhantomData,
        }
    }
}

pub struct AsciiValReader {}

impl AsciiValReader {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

#[derive(Error, Debug)]
pub(crate) enum ReadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse int error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("Parse float error: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),

    #[error("No property found")]
    NoPropertyFound,
}

pub trait ScalarReader {
    fn read_i8(reader: impl Read) -> Result<i8, ReadError>;
    fn read_u8(reader: impl Read) -> Result<u8, ReadError>;
    fn read_i16(reader: impl Read) -> Result<i16, ReadError>;
    fn read_u16(reader: impl Read) -> Result<u16, ReadError>;
    fn read_i32(reader: impl Read) -> Result<i32, ReadError>;
    fn read_u32(reader: impl Read) -> Result<u32, ReadError>;
    fn read_f32(reader: impl Read) -> Result<f32, ReadError>;
    fn read_f64(reader: impl Read) -> Result<f64, ReadError>;
}

impl<E: ByteOrder> ScalarReader for BinValReader<E> {
    fn read_i8(mut reader: impl Read) -> Result<i8, ReadError> {
        Ok(reader.read_i8()?)
    }

    fn read_u8(mut reader: impl Read) -> Result<u8, ReadError> {
        Ok(reader.read_u8()?)
    }

    fn read_i16(mut reader: impl Read) -> Result<i16, ReadError> {
        Ok(reader.read_i16::<E>()?)
    }

    fn read_u16(mut reader: impl Read) -> Result<u16, ReadError> {
        Ok(reader.read_u16::<E>()?)
    }

    fn read_i32(mut reader: impl Read) -> Result<i32, ReadError> {
        Ok(reader.read_i32::<E>()?)
    }

    fn read_u32(mut reader: impl Read) -> Result<u32, ReadError> {
        Ok(reader.read_u32::<E>()?)
    }

    fn read_f32(mut reader: impl Read) -> Result<f32, ReadError> {
        Ok(reader.read_f32::<E>()?)
    }

    fn read_f64(mut reader: impl Read) -> Result<f64, ReadError> {
        Ok(reader.read_f64::<E>()?)
    }
}

impl ScalarReader for AsciiValReader {
    fn read_i8(reader: impl Read) -> Result<i8, ReadError> {
        Ok(Self::read_ascii_token(reader)?.parse::<i8>()?)
    }

    fn read_u8(reader: impl Read) -> Result<u8, ReadError> {
        Ok(Self::read_ascii_token(reader)?.parse::<u8>()?)
    }

    fn read_i16(reader: impl Read) -> Result<i16, ReadError> {
        Ok(Self::read_ascii_token(reader)?.parse::<i16>()?)
    }

    fn read_u16(reader: impl Read) -> Result<u16, ReadError> {
        Ok(Self::read_ascii_token(reader)?.parse::<u16>()?)
    }

    fn read_i32(reader: impl Read) -> Result<i32, ReadError> {
        Ok(Self::read_ascii_token(reader)?.parse::<i32>()?)
    }

    fn read_u32(reader: impl Read) -> Result<u32, ReadError> {
        Ok(Self::read_ascii_token(reader)?.parse::<u32>()?)
    }

    fn read_f32(reader: impl Read) -> Result<f32, ReadError> {
        Ok(Self::read_ascii_token(reader)?.parse::<f32>()?)
    }

    fn read_f64(reader: impl Read) -> Result<f64, ReadError> {
        Ok(Self::read_ascii_token(reader)?.parse::<f64>()?)
    }
}

impl AsciiValReader {
    fn read_ascii_token(mut reader: impl Read) -> Result<String, ReadError> {
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
                Err(e) => return Err(ReadError::Io(e)),
            }
        }

        if token.is_empty() {
            return Err(ReadError::NoPropertyFound);
        }

        Ok(token)
    }
}
