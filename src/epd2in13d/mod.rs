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

    fn sleep(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
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
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        assert!(buffer.len() == buffer_len(WIDTH as usize, HEIGHT as usize));
	    todo!();

        /*
        self.set_ram_area(spi, 0, 0, WIDTH - 1, HEIGHT - 1)?;
        self.set_ram_address_counters(spi, delay, 0, 0)?;

        self.cmd_with_data(spi, Command::WriteRam, buffer)?;

        if self.refresh == RefreshLut::Full {
            // Always keep the base buffer equal to current if not doing partial refresh.
            self.set_ram_area(spi, 0, 0, WIDTH - 1, HEIGHT - 1)?;
            self.set_ram_address_counters(spi, delay, 0, 0)?;

            self.cmd_with_data(spi, Command::WriteRamRed, buffer)?;
        }
        */
        Ok(())
    }

    /// Updating only a part of the frame is not supported when using the
    /// partial refresh feature. The function will panic if called when set to
    /// use partial refresh.
    fn update_partial_frame(
        &mut self,
        spi: &mut SPI,
        delay: &mut DELAY,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        assert!((width * height / 8) as usize == buffer.len());

        // This should not be used when doing partial refresh. The RAM_RED must
        // be updated with the last buffer having been displayed. Doing partial
        // update directly in RAM makes this update impossible (we can't read
        // RAM content). Using this function will most probably make the actual
        // display incorrect as the controler will compare with something
        // incorrect.
        assert!(self.refresh == RefreshLut::Full);

        /*
        self.set_ram_area(spi, x, y, x + width, y + height)?;
        self.set_ram_address_counters(spi, delay, x, y)?;

        self.cmd_with_data(spi, Command::WriteRam, buffer)?;

        if self.refresh == RefreshLut::Full {
            // Always keep the base buffer equals to current if not doing partial refresh.
            self.set_ram_area(spi, x, y, x + width, y + height)?;
            self.set_ram_address_counters(spi, delay, x, y)?;

            self.cmd_with_data(spi, Command::WriteRamRed, buffer)?;
        }
        */

        Ok(())
    }

    /// Never use directly this function when using partial refresh, or also
    /// keep the base buffer in syncd using `set_partial_base_buffer` function.
    fn display_frame(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
	    todo!();
        /*
        if self.refresh == RefreshLut::Full {
            self.set_display_update_control_2(
                spi,
                DisplayUpdateControl2::new()
                    .enable_clock()
                    .enable_analog()
                    .display()
                    .disable_analog()
                    .disable_clock(),
            )?;
        } else {
            self.set_display_update_control_2(spi, DisplayUpdateControl2::new().display())?;
        }
        self.command(spi, Command::MasterActivation)?;
        self.wait_until_idle(spi, delay)?;
        */

        Ok(())
    }

    fn update_and_display_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        /*
        let mut buffer = [0_u8; BUF_LEN as usize];
        for ea in 0..BUF_LEN {
	        buffer[ea as usize] = ea as u8;
        }
        */

        let color = self.background_color.get_byte_value();
        const BUF_LEN: u32 = buffer_len(WIDTH as usize, HEIGHT as usize) as u32;
        self.interface.cmd(spi, Command::DisplayStartTransmission1)?;
        self.interface.data_x_times(spi, color, BUF_LEN)?;
        //self.interface.cmd_with_data(spi, Command::DisplayStartTransmission1, &buffer)?;
        self.interface.cmd_with_data(spi, Command::DisplayStartTransmission2, &buffer)?;
        self.set_lut(spi, delay, None)?;
        self.turn_on_display(spi, delay)?;
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
        /*

        self.set_ram_area(spi, 0, 0, WIDTH - 1, HEIGHT - 1)?;
        self.set_ram_address_counters(spi, delay, 0, 0)?;

        self.command(spi, Command::WriteRam)?;
        self.interface.data_x_times(
            spi,
            color,
            buffer_len(WIDTH as usize, HEIGHT as usize) as u32,
        )?;

        // Always keep the base buffer equals to current if not doing partial refresh.
        if self.refresh == RefreshLut::Full {
            self.set_ram_area(spi, 0, 0, WIDTH - 1, HEIGHT - 1)?;
            self.set_ram_address_counters(spi, delay, 0, 0)?;

            self.command(spi, Command::WriteRamRed)?;
            self.interface.data_x_times(
                spi,
                color,
                buffer_len(WIDTH as usize, HEIGHT as usize) as u32,
            )?;
        }
        */
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
        self.interface.wait_until_idle_with_cmd(spi, delay, IS_BUSY_LOW, Command::GetStatus);
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
