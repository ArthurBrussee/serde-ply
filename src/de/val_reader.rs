use std::io::Read;
use std::marker::PhantomData;

use byteorder::ByteOrder;
use byteorder::ReadBytesExt;

use crate::PlyError;

pub struct BinValReader<R: Read, E: ByteOrder> {
    reader: R,
    _endian: PhantomData<E>,
}
impl<R: Read, E: ByteOrder> BinValReader<R, E> {
    pub(crate) fn new(reader: R) -> Self {
        Self {
            reader,
            _endian: PhantomData,
        }
    }
}

pub struct AsciiValReader<R: Read> {
    reader: R,
}

impl<R: Read> AsciiValReader<R> {
    pub(crate) fn new(reader: R) -> Self {
        Self { reader }
    }
}

pub trait ScalarReader {
    fn read_i8(&mut self) -> Result<i8, PlyError>;
    fn read_u8(&mut self) -> Result<u8, PlyError>;
    fn read_i16(&mut self) -> Result<i16, PlyError>;
    fn read_u16(&mut self) -> Result<u16, PlyError>;
    fn read_i32(&mut self) -> Result<i32, PlyError>;
    fn read_u32(&mut self) -> Result<u32, PlyError>;
    fn read_f32(&mut self) -> Result<f32, PlyError>;
    fn read_f64(&mut self) -> Result<f64, PlyError>;
}

impl<R: Read, E: ByteOrder> ScalarReader for BinValReader<R, E> {
    fn read_i8(&mut self) -> Result<i8, PlyError> {
        Ok(self.reader.read_i8()?)
    }

    fn read_u8(&mut self) -> Result<u8, PlyError> {
        Ok(self.reader.read_u8()?)
    }

    fn read_i16(&mut self) -> Result<i16, PlyError> {
        Ok(self.reader.read_i16::<E>()?)
    }

    fn read_u16(&mut self) -> Result<u16, PlyError> {
        Ok(self.reader.read_u16::<E>()?)
    }

    fn read_i32(&mut self) -> Result<i32, PlyError> {
        Ok(self.reader.read_i32::<E>()?)
    }

    fn read_u32(&mut self) -> Result<u32, PlyError> {
        Ok(self.reader.read_u32::<E>()?)
    }

    fn read_f32(&mut self) -> Result<f32, PlyError> {
        Ok(self.reader.read_f32::<E>()?)
    }

    fn read_f64(&mut self) -> Result<f64, PlyError> {
        Ok(self.reader.read_f64::<E>()?)
    }
}

impl<R: Read> ScalarReader for AsciiValReader<R> {
    fn read_i8(&mut self) -> Result<i8, PlyError> {
        Ok(read_ascii_token(&mut self.reader)?.parse::<i8>()?)
    }

    fn read_u8(&mut self) -> Result<u8, PlyError> {
        Ok(read_ascii_token(&mut self.reader)?.parse::<u8>()?)
    }

    fn read_i16(&mut self) -> Result<i16, PlyError> {
        Ok(read_ascii_token(&mut self.reader)?.parse::<i16>()?)
    }

    fn read_u16(&mut self) -> Result<u16, PlyError> {
        Ok(read_ascii_token(&mut self.reader)?.parse::<u16>()?)
    }

    fn read_i32(&mut self) -> Result<i32, PlyError> {
        Ok(read_ascii_token(&mut self.reader)?.parse::<i32>()?)
    }

    fn read_u32(&mut self) -> Result<u32, PlyError> {
        Ok(read_ascii_token(&mut self.reader)?.parse::<u32>()?)
    }

    fn read_f32(&mut self) -> Result<f32, PlyError> {
        Ok(read_ascii_token(&mut self.reader)?.parse::<f32>()?)
    }

    fn read_f64(&mut self) -> Result<f64, PlyError> {
        Ok(read_ascii_token(&mut self.reader)?.parse::<f64>()?)
    }
}

fn read_ascii_token<R: Read>(reader: &mut R) -> Result<String, PlyError> {
    let mut token = String::new();
    let mut in_token = false;

    loop {
        let mut byte = [0u8; 1];
        match reader.read_exact(&mut byte) {
            Ok(_) => {
                let ch = byte[0] as char;
                if ch.is_ascii_whitespace() {
                    if in_token || ch == '\n' {
                        break;
                    }
                } else {
                    in_token = true;
                    token.push(ch);
                }
            }
            Err(e) => return Err(PlyError::Io(e)),
        }
    }

    if !in_token {
        return Err(PlyError::NoTokenFound);
    }

    Ok(token)
}
