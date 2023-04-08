//! SPI Commands for the Waveshare 2.13" (D)

use crate::traits;

extern crate bit_field;
use bit_field::BitField;

/// Epd2in13d
///
/// For more infos about the addresses and what they are doing look into the pdfs
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) enum Command {
	PanelSetting = 0x00,
	PowerSetting = 0x01,
	PowerOff = 0x02,
	PowerOffSequenceSetting = 0x03,
	PowerOn = 0x04,
	PowerOnMeasure = 0x05,
	BoosterSoftStart = 0x06,
	DeepSleep = 0x07,
	DisplayStartTransmission1 = 0x10,
	DataStop = 0x11,
	DisplayRefresh = 0x12,
	DisplayStartTransmission2 = 0x13,
	AutoSequence = 0x17,
	VcomLut = 0x20,
	WhiteToWhiteLut = 0x21,
	BlackToWhiteLut = 0x22,
	WhiteToBlackLut = 0x23,
	BlackToBlackLut = 0x24,
	LutOption = 0x2a,
	PllControl = 0x30,
	TemperatureSensorCalibration = 0x40,
	TemperatureSensorSelection = 0x41,
	TemperatureSensorWrite = 0x42,
	TemperatureSensorRead = 0x43,
	PanelBreakCheck = 0x44,
	VcomAndDataIntervalSetting = 0x50,
	LowerPowerDetection = 0x51,
	TconSetting = 0x60,
	ResolutionSetting = 0x61,
	GateSourceStartSetting = 0x65,
	Revision = 0x70,
	GetStatus = 0x71,
	AutoMeasurementVcom = 0x80,
	ReadVcomValue = 0x81,
	VcmDcSetting = 0x82,
	PartialWindow = 0x90,
	PartialIn = 0x91,
	PartialOut = 0x92,
	ProgramMode = 0xa0,
	ActiveProgramming = 0xa1,
	ReadOtp = 0xa2,
	CascadeSetting = 0xe0,
	PowerSaving = 0xe3,
	LvdVoltageSelect = 0xe4,
	ForceTemperature = 0xe5,
}
