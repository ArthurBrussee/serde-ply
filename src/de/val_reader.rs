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
    row_ended: bool,
    reader: R,
}

impl<R: Read> AsciiValReader<R> {
    pub(crate) fn new(reader: R) -> Self {
        Self {
            row_ended: false,
            reader,
        }
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

    fn read_row_end(&mut self) -> Result<(), PlyError>;
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

    fn read_row_end(&mut self) -> Result<(), PlyError> {
        Ok(())
    }
}

impl<R: Read> ScalarReader for AsciiValReader<R> {
    fn read_i8(&mut self) -> Result<i8, PlyError> {
        Ok(self.read_ascii_token()?.parse::<i8>()?)
    }

    fn read_u8(&mut self) -> Result<u8, PlyError> {
        Ok(self.read_ascii_token()?.parse::<u8>()?)
    }

    fn read_i16(&mut self) -> Result<i16, PlyError> {
        Ok(self.read_ascii_token()?.parse::<i16>()?)
    }

    fn read_u16(&mut self) -> Result<u16, PlyError> {
        Ok(self.read_ascii_token()?.parse::<u16>()?)
    }

    fn read_i32(&mut self) -> Result<i32, PlyError> {
        Ok(self.read_ascii_token()?.parse::<i32>()?)
    }

    fn read_u32(&mut self) -> Result<u32, PlyError> {
        Ok(self.read_ascii_token()?.parse::<u32>()?)
    }

    fn read_f32(&mut self) -> Result<f32, PlyError> {
        Ok(self.read_ascii_token()?.parse::<f32>()?)
    }

    fn read_f64(&mut self) -> Result<f64, PlyError> {
        Ok(self.read_ascii_token()?.parse::<f64>()?)
    }

    fn read_row_end(&mut self) -> Result<(), PlyError> {
        if !self.row_ended {
            loop {
                let mut byte = [0u8; 1];
                match self.reader.read_exact(&mut byte) {
                    Ok(_) => {
                        let ch = byte[0] as char;
                        if ch.is_ascii_whitespace() {
                            if ch == '\n' {
                                return Ok(());
                            }
                        } else {
                            return Err(PlyError::TooManyProperties);
                        }
                    }
                    Err(e) => return Err(PlyError::Io(e)),
                }
            }
        }

        self.row_ended = false;

        Ok(())
    }
}

impl<R: Read> AsciiValReader<R> {
    fn read_ascii_token(&mut self) -> Result<String, PlyError> {
        let mut token = String::new();

        loop {
            let mut byte = [0u8; 1];
            match self.reader.read_exact(&mut byte) {
                Ok(_) => {
                    let ch = byte[0] as char;
                    if ch.is_ascii_whitespace() {
                        if !token.is_empty() {
                            if ch == '\n' {
                                self.row_ended = true;
                            }

                            break;
                        }
                    } else {
                        token.push(ch);
                    }
                }
                Err(e) => return Err(PlyError::Io(e)),
            }
        }

        if token.is_empty() {
            return Err(PlyError::NoPropertyFound);
        }

        Ok(token)
    }
}
