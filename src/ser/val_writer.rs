use std::io::Write;
use std::marker::PhantomData;

use byteorder::ByteOrder;
use byteorder::WriteBytesExt;

use crate::PlyError;

pub struct BinValWriter<W: Write, E: ByteOrder> {
    writer: W,
    _endian: PhantomData<E>,
}

impl<W: Write, E: ByteOrder> BinValWriter<W, E> {
    pub(crate) fn new(writer: W) -> Self {
        Self {
            writer,
            _endian: PhantomData,
        }
    }
}

pub struct AsciiValWriter<W: Write> {
    writer: W,
}

impl<W: Write> AsciiValWriter<W> {
    pub(crate) fn new(writer: W) -> Self {
        Self { writer }
    }
}

pub trait ScalarWriter {
    fn write_i8(&mut self, val: i8) -> Result<(), PlyError>;
    fn write_u8(&mut self, val: u8) -> Result<(), PlyError>;
    fn write_i16(&mut self, val: i16) -> Result<(), PlyError>;
    fn write_u16(&mut self, val: u16) -> Result<(), PlyError>;
    fn write_i32(&mut self, val: i32) -> Result<(), PlyError>;
    fn write_u32(&mut self, val: u32) -> Result<(), PlyError>;
    fn write_f32(&mut self, val: f32) -> Result<(), PlyError>;
    fn write_f64(&mut self, val: f64) -> Result<(), PlyError>;

    fn write_row_end(&mut self) -> Result<(), PlyError>;
}

impl<W: Write, E: ByteOrder> ScalarWriter for BinValWriter<W, E> {
    fn write_i8(&mut self, val: i8) -> Result<(), PlyError> {
        Ok(self.writer.write_i8(val)?)
    }

    fn write_u8(&mut self, val: u8) -> Result<(), PlyError> {
        Ok(self.writer.write_u8(val)?)
    }

    fn write_i16(&mut self, val: i16) -> Result<(), PlyError> {
        Ok(self.writer.write_i16::<E>(val)?)
    }

    fn write_u16(&mut self, val: u16) -> Result<(), PlyError> {
        Ok(self.writer.write_u16::<E>(val)?)
    }

    fn write_i32(&mut self, val: i32) -> Result<(), PlyError> {
        Ok(self.writer.write_i32::<E>(val)?)
    }

    fn write_u32(&mut self, val: u32) -> Result<(), PlyError> {
        Ok(self.writer.write_u32::<E>(val)?)
    }

    fn write_f32(&mut self, val: f32) -> Result<(), PlyError> {
        Ok(self.writer.write_f32::<E>(val)?)
    }

    fn write_f64(&mut self, val: f64) -> Result<(), PlyError> {
        Ok(self.writer.write_f64::<E>(val)?)
    }

    fn write_row_end(&mut self) -> Result<(), PlyError> {
        Ok(())
    }
}

impl<W: Write> ScalarWriter for AsciiValWriter<W> {
    fn write_i8(&mut self, val: i8) -> Result<(), PlyError> {
        write!(self.writer, "{} ", val)?;
        Ok(())
    }

    fn write_u8(&mut self, val: u8) -> Result<(), PlyError> {
        write!(self.writer, "{} ", val)?;
        Ok(())
    }

    fn write_i16(&mut self, val: i16) -> Result<(), PlyError> {
        write!(self.writer, "{} ", val)?;
        Ok(())
    }

    fn write_u16(&mut self, val: u16) -> Result<(), PlyError> {
        write!(self.writer, "{} ", val)?;
        Ok(())
    }

    fn write_i32(&mut self, val: i32) -> Result<(), PlyError> {
        write!(self.writer, "{} ", val)?;
        Ok(())
    }

    fn write_u32(&mut self, val: u32) -> Result<(), PlyError> {
        write!(self.writer, "{} ", val)?;
        Ok(())
    }

    fn write_f32(&mut self, val: f32) -> Result<(), PlyError> {
        write!(self.writer, "{} ", val)?;
        Ok(())
    }

    fn write_f64(&mut self, val: f64) -> Result<(), PlyError> {
        write!(self.writer, "{} ", val)?;
        Ok(())
    }

    fn write_row_end(&mut self) -> Result<(), PlyError> {
        writeln!(self.writer)?;
        Ok(())
    }
}
