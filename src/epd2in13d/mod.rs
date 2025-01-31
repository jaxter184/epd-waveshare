//! A Driver for the Waveshare 2.13" E-Ink Display (V2) via SPI
//!
//! # References
//!
//! - [Waveshare product page](https://www.waveshare.com/wiki/2.13inch_e-Paper_HAT)
//! - [Waveshare C driver](https://github.com/waveshare/e-Paper/blob/master/RaspberryPi%26JetsonNano/c/lib/e-Paper/EPD_2in13_V2.c)
//! - [Waveshare Python driver](https://github.com/waveshare/e-Paper/blob/master/RaspberryPi%26JetsonNano/python/lib/waveshare_epd/epd2in13_V2.py)
//! - [Controller Datasheet SS1780](http://www.e-paper-display.com/download_detail/downloadsId=682.html)
//!

use embedded_hal::{
    blocking::{delay::*, spi::Write},
    digital::v2::{InputPin, OutputPin},
};

use crate::buffer_len;
use crate::color::Color;
use crate::interface::DisplayInterface;
use crate::traits::{InternalWiAdditions, RefreshLut, WaveshareDisplay};

pub(crate) mod command;
use command::Command;

pub(crate) mod constants;
use self::constants::{
    LUT_FULL_VCOM, LUT_FULL_WW, LUT_FULL_BW, LUT_FULL_WB, LUT_FULL_BB,
    LUT_PART_VCOM, LUT_PART_WW, LUT_PART_BW, LUT_PART_WB, LUT_PART_BB,
};

/// Full size buffer for use with the 2in13 v2 EPD
#[cfg(feature = "graphics")]
pub type Display2in13 = crate::graphics::Display<
    WIDTH,
    HEIGHT,
    false,
    { buffer_len(WIDTH as usize, HEIGHT as usize) },
    Color,
>;

/// Width of the display.
pub const WIDTH: u32 = 104;

/// Height of the display
pub const HEIGHT: u32 = 212;

/// Default Background Color
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::White;
const IS_BUSY_LOW: bool = false;

/// Epd2in13 (V2) driver
///
pub struct Epd2in13<SPI, CS, BUSY, DC, RST, DELAY> {
    /// Connection Interface
    interface: DisplayInterface<SPI, CS, BUSY, DC, RST, DELAY>,

    /// Background Color
    background_color: Color,
    refresh: RefreshLut,
}

impl<SPI, CS, BUSY, DC, RST, DELAY> Epd2in13<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayUs<u32>,
{
    fn turn_on_display(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.interface.cmd(spi, Command::DisplayRefresh)?;
        delay.delay_us(100000); // can apparently be as low as 200us
        self.wait_until_idle(spi, delay)
    }
}

impl<SPI, CS, BUSY, DC, RST, DELAY> InternalWiAdditions<SPI, CS, BUSY, DC, RST, DELAY>
    for Epd2in13<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayUs<u32>,
{
    fn init(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        // HW reset
        self.interface.reset(delay, 10_000, 10_000);

        self.interface.cmd_with_data(spi, Command::PowerSetting, &[0x03, 0x00, 0x2b, 0x2b, 0x03])?;
        self.interface.cmd_with_data(spi, Command::BoosterSoftStart, &[0x17, 0x17, 0x17])?;
        self.interface.cmd(spi, Command::PowerOn)?;
        self.wait_until_idle(spi, delay)?;
        self.interface.cmd_with_data(spi, Command::PanelSetting, &[0xbf, 0x0e])?;
        self.interface.cmd_with_data(spi, Command::PllControl, &[0x3a])?;
        self.interface.cmd_with_data(spi, Command::ResolutionSetting, &[
            WIDTH as u8,
            ((HEIGHT >> 8) & 0xff) as u8,
            (HEIGHT & 0xff) as u8,
        ])?;
        self.interface.cmd_with_data(spi, Command::VcmDcSetting, &[0x28])?;

        self.wait_until_idle(spi, delay)?;
        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST, DELAY> WaveshareDisplay<SPI, CS, BUSY, DC, RST, DELAY>
    for Epd2in13<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayUs<u32>,
{
    type DisplayColor = Color;
    fn new(
        spi: &mut SPI,
        cs: CS,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
        delay_us: Option<u32>,
    ) -> Result<Self, SPI::Error> {
        let mut epd = Epd2in13 {
            interface: DisplayInterface::new(cs, busy, dc, rst, delay_us),
            //sleep_mode: DeepSleepMode::Mode1,
            background_color: DEFAULT_BACKGROUND_COLOR,
            refresh: RefreshLut::Full,
        };

        epd.init(spi, delay)?;
        Ok(epd)
    }

    fn wake_up(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.init(spi, delay)
    }

    fn sleep(&mut self, _spi: &mut SPI, _delay: &mut DELAY) -> Result<(), SPI::Error> {
        /*
        self.wait_until_idle(spi, delay)?;

        // All sample code enables and disables analog/clocks...
        self.set_display_update_control_2(
            spi,
            DisplayUpdateControl2::new()
                .enable_analog()
                .enable_clock()
                .disable_analog()
                .disable_clock(),
        )?;
        self.command(spi, Command::MasterActivation)?;

        self.set_sleep_mode(spi, self.sleep_mode)?;
        */
        Ok(())
    }

    fn update_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        _delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        assert!(buffer.len() == buffer_len(WIDTH as usize, HEIGHT as usize));

        self.refresh = RefreshLut::Full;

        let color = self.background_color.get_byte_value();
        const BUF_LEN: u32 = buffer_len(WIDTH as usize, HEIGHT as usize) as u32;
        self.interface.cmd(spi, Command::DisplayStartTransmission1)?;
        self.interface.data_x_times(spi, color, BUF_LEN)?;
        self.interface.cmd_with_data(spi, Command::DisplayStartTransmission2, &buffer)?;
        Ok(())
    }

    /// Updating only a part of the frame is not supported when using the
    /// partial refresh feature. The function will panic if called when set to
    /// use partial refresh.
    fn update_partial_frame(
        &mut self,
        spi: &mut SPI,
        _delay: &mut DELAY,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        assert!((width * height / 8) as usize == buffer.len());
        assert!(x % 8 == 0);
        assert!(width % 8 == 0);

        self.refresh = RefreshLut::Quick;

        self.interface.cmd(spi, Command::PartialIn)?;
        self.interface.cmd(spi, Command::PartialWindow)?;
        self.interface.data(spi, &[x as u8])?;
        self.interface.data(spi, &[(x + width - 1) as u8])?;
        self.interface.data(spi, &[(y / 256) as u8])?;
        self.interface.data(spi, &[(y % 256) as u8])?;
        self.interface.data(spi, &[((y + height) / 256) as u8])?;
        self.interface.data(spi, &[((y + height) % 256 - 1) as u8])?;
        self.interface.data(spi, &[0x28])?;
        self.interface.cmd(spi, Command::DisplayStartTransmission1)?;
        for ea_byte in buffer {
            self.interface.data(spi, &[!ea_byte])?;
        }
        self.interface.cmd(spi, Command::DisplayStartTransmission2)?;
        self.interface.data(spi, &buffer)?;

        Ok(())
    }

    fn display_frame(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.set_lut(spi, delay, Some(self.refresh))?;
        self.turn_on_display(spi, delay)?;
        Ok(())
    }

    fn update_and_display_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.update_frame(spi, buffer, delay)?;
        self.display_frame(spi, delay)?;
        Ok(())
    }

    fn clear_frame(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        let color = self.background_color.get_byte_value();
        const BUF_LEN: u32 = buffer_len(WIDTH as usize, HEIGHT as usize) as u32;
        self.interface.cmd(spi, Command::DisplayStartTransmission1)?;
        self.interface.data_x_times(spi, color, BUF_LEN)?;
        self.interface.cmd(spi, Command::DisplayStartTransmission2)?;
        self.interface.data_x_times(spi, !color, BUF_LEN)?;
        self.set_lut(spi, delay, None)?;
        self.turn_on_display(spi, delay)?;
        Ok(())
    }

    fn set_background_color(&mut self, background_color: Color) {
        self.background_color = background_color;
    }

    fn background_color(&self) -> &Color {
        &self.background_color
    }

    fn width(&self) -> u32 {
        WIDTH
    }

    fn height(&self) -> u32 {
        HEIGHT
    }

    fn set_lut(
        &mut self,
        spi: &mut SPI,
        _delay: &mut DELAY,
        refresh_rate: Option<RefreshLut>,
    ) -> Result<(), SPI::Error> {
        let (vcom, ww, bw, wb, bb) = match refresh_rate {
            Some(RefreshLut::Full) | None =>
                (&LUT_FULL_VCOM, &LUT_FULL_WW, &LUT_FULL_BW, &LUT_FULL_WB, &LUT_FULL_BB),
            Some(RefreshLut::Quick) => {
                self.interface.cmd_with_data(spi, Command::VcmDcSetting, &[0x00])?;
                (&LUT_PART_VCOM, &LUT_PART_WW, &LUT_PART_BW, &LUT_PART_WB, &LUT_PART_BB)
            }
        };

        self.interface.cmd_with_data(spi, Command::VcomAndDataIntervalSetting, &[0xb7])?;
        self.interface.cmd_with_data(spi, Command::VcomLut, vcom)?;
        self.interface.cmd_with_data(spi, Command::WhiteToWhiteLut, ww)?;
        self.interface.cmd_with_data(spi, Command::BlackToWhiteLut, bw)?;
        self.interface.cmd_with_data(spi, Command::WhiteToBlackLut, wb)?;
        self.interface.cmd_with_data(spi, Command::BlackToBlackLut, bb)
    }

    fn wait_until_idle(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.interface.wait_until_idle_with_cmd(spi, delay, IS_BUSY_LOW, Command::GetStatus)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epd_size() {
        assert_eq!(WIDTH, 104);
        assert_eq!(HEIGHT, 212);
        assert_eq!(DEFAULT_BACKGROUND_COLOR, Color::White);
    }
}
