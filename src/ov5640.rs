use embedded_hal::i2c::I2c;

use core::convert::TryInto;

use crate::constants::*;

#[cfg(feature = "defmt-03")]
use crate::defmt;

#[derive(Debug)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum SccbError<I2CE> {
    I2c(I2CE),
    InvalidId(u8),
}

pub struct Ov5640<I2C> {
    i2c: I2C,
}

pub enum Resolution {
    Qcifz176_144,
    Qvga320_240,
    Vga640_480,
    Ntsc720_480,
    Pal720_576,
    Xga1024_768,
    P720_1280_720,
    P1080_1920_1080,
    Qsxga2592_1944,
}

pub enum Format {
    Raw(RawOrder),
    Rgb565(Rgb565Order),
    Yuv422(Yuv422Order),
    Jpeg,
}

pub enum RawOrder {
    SBGGR8, // BGBG... / GRGR...0x0,
    SGBRG8, // GBGB... / RGRG...0x1,
    SGRBG8, // GRGR... / BGBG...0x2,
    SRGGB8, // RGRG... / GBGB...0x3
}

pub enum Rgb565Order {
    Bggr,
    Rggb,
    Grrb,
    Brrg,
    Gbbr,
    Rbbg,
}

pub enum Yuv422Order {
    Yuyv,
    Yvyu,
    Uyvy,
    Vyuy,
}

impl Format {
    fn format_bits(&self) -> u8 {
        match self {
            Format::Raw(order) => order.to_hex(),
            Format::Rgb565(order) => 0x60 | order.to_hex(),
            Format::Yuv422(order) => 0x30 | order.to_hex(),
            Format::Jpeg => 0x30 | 0,
        }
    }

    fn mux_bits(&self) -> u8 {
        match self {
            Format::Raw(_) => OV5640_FMT_MUX_RAW_DPC,
            Format::Rgb565(_) => OV5640_FMT_MUX_RGB,
            Format::Yuv422(_) => OV5640_FMT_MUX_YUV422,
            Format::Jpeg => OV5640_FMT_MUX_JPEG,
        }
    }
}

impl RawOrder {
    fn to_hex(&self) -> u8 {
        match self {
            RawOrder::SBGGR8 => 0,
            RawOrder::SGBRG8 => 1,
            RawOrder::SGRBG8 => 2,
            RawOrder::SRGGB8 => 3,
        }
    }
}

impl Rgb565Order {
    fn to_hex(&self) -> u8 {
        match self {
            Rgb565Order::Bggr => 0,
            Rgb565Order::Rggb => 1,
            Rgb565Order::Grrb => 2,
            Rgb565Order::Brrg => 3,
            Rgb565Order::Gbbr => 4,
            Rgb565Order::Rbbg => 5,
        }
    }
}

impl Yuv422Order {
    fn to_hex(&self) -> u8 {
        match self {
            Yuv422Order::Yuyv => 0,
            Yuv422Order::Yvyu => 1,
            Yuv422Order::Uyvy => 2,
            Yuv422Order::Vyuy => 3,
        }
    }
}

impl<I2C, E> Ov5640<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn new(i2c: I2C) -> Self
    where
        I2C: I2c<Error = E>,
    {
        Ov5640 { i2c }
    }

    pub fn init(&mut self, format: Format, resolution: Resolution) -> Result<(), SccbError<E>> {
        let slave_id = self.read_reg(OV5640_REG_ID)?;
        if slave_id != OV5640_ID {
            return Err(SccbError::InvalidId(slave_id));
        }

        for register in OV5640_INITIAL_SETTINGS.iter() {
            self.write_reg(register.0, register.1)?;
        }

        for register in match resolution {
            Resolution::Qcifz176_144 => QCIF_176_144.iter(),
            Resolution::Qvga320_240 => QVGA_320_240.iter(),
            Resolution::Vga640_480 => VGA_640_480.iter(),
            Resolution::Ntsc720_480 => NTSC_720_480.iter(),
            Resolution::Pal720_576 => PAL_720_576.iter(),
            Resolution::Xga1024_768 => XGA_1024_768.iter(),
            Resolution::P720_1280_720 => P720_1280_720.iter(),
            Resolution::P1080_1920_1080 => P1080_1920_1080.iter(),
            Resolution::Qsxga2592_1944 => QSXGA_2592_1944.iter(),
        } {
            self.write_reg(register.0, register.1)?;
        }

        // configure the output format
        self.write_reg(OV5640_REG_FORMAT_00, format.format_bits())?;
        self.write_reg(OV5640_REG_ISP_FORMAT_MUX_CTRL, format.mux_bits())?;
        if let Format::Jpeg = format {
            for register in OV5640_FMT_JPEG_EXTRA.iter() {
                self.write_reg(register.0, register.1)?;
            }
        }

        Ok(())
    }

    fn write_reg(&mut self, reg: u16, val: u8) -> Result<(), SccbError<E>> {
        self.i2c
            .write(
                OV5640_ADDR,
                &[
                    (reg >> 8).try_into().unwrap(),
                    (reg & 0xff).try_into().unwrap(),
                    val,
                ],
            )
            .map_err(|e| SccbError::I2c(e))
    }

    fn read_reg(&mut self, reg: u16) -> Result<u8, SccbError<E>> {
        self.i2c
            .write(
                OV5640_ADDR,
                &[
                    (reg >> 8).try_into().unwrap(),
                    (reg & 0xff).try_into().unwrap(),
                ],
            )
            .map_err(|e| SccbError::I2c(e))?;

        let mut buf: [u8; 1] = [0];

        self.i2c
            .read(OV5640_ADDR, &mut buf)
            .map_err(|e| SccbError::I2c(e))?;

        Ok(buf[0])
    }

    pub fn free(self) -> I2C {
        self.i2c
    }
}
