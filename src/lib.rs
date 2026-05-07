use std::io::Write;
use std::time::Duration;

use serial::SerialPort;
use serial::unix::TTYPort;

#[derive(Default)]
pub enum Region {
    EU433,
    #[default]
    EU868,
    US,
}

impl Region {
    pub fn freq_start(&self) -> u32 {
        match self {
            Region::EU433 => 433_000_000,
            Region::EU868 => 869_400_000,
            Region::US => 902_000_000,
        }
    }

    pub fn freq_end(&self) -> u32 {
        match self {
            Region::EU433 => 434_000_000,
            Region::EU868 => 869_650_000,
            Region::US => 928_000_000,
        }
    }

    pub fn tx_power_limit(&self) -> u8 {
        match self {
            Region::EU433 => 10,
            Region::EU868 => 27,
            Region::US => 30,
        }
    }
}

#[derive(Default)]
pub enum Preset {
    #[default]
    LongFast,
}

impl Preset {
    fn bandwidth(&self) -> u32 {
        match self {
            Preset::LongFast => 250_000,
        }
    }
}

#[repr(u8)]
#[allow(unused)]
enum KISS {
    FEND = 0xC0,
    FESC = 0xDB,
    TFEND = 0xDC,
    TFESC = 0xDD,
}

#[allow(unused)]
impl KISS {
    pub fn escape(bytes: &[u8]) -> Vec<u8> {
        let bytes = Self::replace(
            bytes,
            &[KISS::FESC as u8],
            &[KISS::FESC as u8, KISS::TFESC as u8],
        );
        let bytes = Self::replace(
            &bytes,
            &[KISS::FEND as u8],
            &[KISS::FEND as u8, KISS::TFEND as u8],
        );
        bytes
    }

    fn replace(source: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
        let mut result = source.to_vec();
        let from_len = from.len();
        let to_len = to.len();

        let mut i = 0;
        while i + from_len <= result.len() {
            if result[i..].starts_with(from) {
                result.splice(i..i + from_len, to.iter().cloned());
                i += to_len;
            } else {
                i += 1;
            }
        }

        result
    }
}

#[repr(u8)]
#[allow(non_camel_case_types, unused)]
enum RNODE {
    CMD_DATA = 0x00,
    CMD_FREQUENCY = 0x01,
    CMD_BANDWIDTH = 0x02,
    CMD_TXPOWER = 0x03,
    CMD_SF = 0x04,
    CMD_CR = 0x05,
    CMD_RADIO_STATE = 0x06,
    CMD_RADIO_LOCK = 0x07,
    CMD_DETECT = 0x08,
    CMD_PROMISC = 0x0E,
    CMD_READY = 0x0F,
    CMD_STAT_RX = 0x21,
    CMD_STAT_TX = 0x22,
    CMD_STAT_RSSI = 0x23,
    CMD_STAT_SNR = 0x24,
    CMD_BLINK = 0x30,
    CMD_RANDOM = 0x40,
    CMD_FW_VERSION = 0x50,
    CMD_ROM_READ = 0x51,
}

#[repr(u8)]
#[allow(unused)]
enum RadioState {
    OFF = 0x00,
    ON = 0x01,
    ASK = 0xFF,
}

pub struct RadioConfig {
    frequency: u32,
    bandwidth: u32,
    sf: u8,
    cr: u8,
    tx_power: u8,
}

impl RadioConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_preset(region: Region, preset: Preset) -> Self {
        Self::default()
            .frequency(region.freq_start())
            .bandwidth(preset.bandwidth())
    }

    pub fn frequency(mut self, frequency: u32) -> Self {
        self.frequency = frequency;
        self
    }

    pub fn bandwidth(mut self, bandwidth: u32) -> Self {
        self.bandwidth = bandwidth;
        self
    }

    pub fn spread_factor(mut self, sf: u8) -> Self {
        self.sf = sf;
        self
    }

    pub fn coding_rate(mut self, cr: u8) -> Self {
        self.cr = cr;
        self
    }

    pub fn tx_power(mut self, tx_power: u8) -> Self {
        self.tx_power = tx_power;
        self
    }
}

impl Default for RadioConfig {
    fn default() -> Self {
        Self::from_preset(Region::default(), Preset::default())
    }
}

pub struct RNodeInterface {
    port: TTYPort,
    config: RadioConfig,
}

impl RNodeInterface {
    pub fn new(port: &str, config: RadioConfig) -> Result<Self, serial::Error> {
        // Initialise serial port.
        let mut port = serial::open(port)?;

        port.reconfigure(&|settings| {
            settings.set_baud_rate(serial::Baud115200)?;
            settings.set_char_size(serial::Bits8);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop1);
            settings.set_flow_control(serial::FlowNone);
            Ok(())
        })?;
        port.set_timeout(Duration::from_millis(1000))?;

        // Initialise radio.
        let mut rnode = Self { port, config };
        rnode.set_frequency()?;
        rnode.set_bandwidth()?;
        rnode.set_tx_power()?;
        rnode.set_spreading_factor()?;
        rnode.set_coding_rate()?;
        rnode.set_radio_state(RadioState::ON)?;

        Ok(rnode)
    }

    fn set_frequency(&mut self) -> Result<(), serial::Error> {
        let mut command = Vec::new();
        command.extend_from_slice(&[KISS::FEND as u8, RNODE::CMD_FREQUENCY as u8]);
        command.extend_from_slice(&self.config.frequency.to_be_bytes());
        command.extend_from_slice(&[KISS::FEND as u8]);

        self.port.write_all(&command)?;
        Ok(())
    }

    fn set_bandwidth(&mut self) -> Result<(), serial::Error> {
        let mut command = Vec::new();
        command.extend_from_slice(&[KISS::FEND as u8, RNODE::CMD_BANDWIDTH as u8]);
        command.extend_from_slice(&self.config.bandwidth.to_be_bytes());
        command.extend_from_slice(&[KISS::FEND as u8]);

        self.port.write_all(&command)?;
        Ok(())
    }

    fn set_spreading_factor(&mut self) -> Result<(), serial::Error> {
        self.port.write_all(&[
            KISS::FEND as u8,
            RNODE::CMD_SF as u8,
            self.config.sf,
            KISS::FEND as u8,
        ])?;
        Ok(())
    }

    fn set_tx_power(&mut self) -> Result<(), serial::Error> {
        self.port.write_all(&[
            KISS::FEND as u8,
            RNODE::CMD_TXPOWER as u8,
            self.config.tx_power,
            KISS::FEND as u8,
        ])?;
        Ok(())
    }

    fn set_coding_rate(&mut self) -> Result<(), serial::Error> {
        self.port.write_all(&[
            KISS::FEND as u8,
            RNODE::CMD_CR as u8,
            self.config.cr,
            KISS::FEND as u8,
        ])?;
        Ok(())
    }

    fn set_radio_state(&mut self, state: RadioState) -> Result<(), serial::Error> {
        self.port.write_all(&[
            KISS::FEND as u8,
            RNODE::CMD_RADIO_STATE as u8,
            state as u8,
            KISS::FEND as u8,
        ])?;
        Ok(())
    }
}
