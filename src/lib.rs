use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serialport::{DataBits, FlowControl, Parity, StopBits};

const BAUD_RATE: u32 = 115_200;
const MTU: usize = 508;

#[derive(Default)]
pub enum Region {
    EU433,
    #[default]
    EU868,
    US,
}

impl Region {
    pub fn min_frequency(&self) -> u32 {
        match self {
            Region::EU433 => 433_000_000,
            Region::EU868 => 869_400_000,
            Region::US => 902_000_000,
        }
    }

    pub fn max_frequency(&self) -> u32 {
        match self {
            Region::EU433 => 434_000_000,
            Region::EU868 => 869_650_000,
            Region::US => 928_000_000,
        }
    }

    pub fn max_tx_power(&self) -> u8 {
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

    fn coding_rate(&self) -> u8 {
        match self {
            Preset::LongFast => 5,
        }
    }

    fn spread_factor(&self) -> u8 {
        match self {
            Preset::LongFast => 11,
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
            .frequency(region.min_frequency())
            .tx_power(region.max_tx_power())
            .bandwidth(preset.bandwidth())
            .spread_factor(preset.spread_factor())
            .coding_rate(preset.coding_rate())
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
    config: RadioConfig,
    tx: mpsc::Sender<Vec<u8>>,
}

enum State {
    Tx,
    Rx,
}

#[derive(Debug)]
pub enum ErrorKind {
    SerialPort,
    Internal,
    PayloadTooLarge,
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub description: String,
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.description
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.description)
    }
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    fn payload_too_large() -> Self {
        Self {
            kind: ErrorKind::PayloadTooLarge,
            description: format!("payload can't be larger than MTU of {}", MTU),
        }
    }
}

impl From<mpsc::SendError<Vec<u8>>> for Error {
    fn from(_error: mpsc::SendError<Vec<u8>>) -> Self {
        Self {
            kind: ErrorKind::Internal,
            description: "internal tx thread shut down".to_string(),
        }
    }
}

impl From<serialport::Error> for Error {
    fn from(error: serialport::Error) -> Self {
        Self {
            kind: ErrorKind::SerialPort,
            description: error.description,
        }
    }
}

impl RNodeInterface {
    pub fn new(port: &str, config: RadioConfig) -> Result<Self, Error> {
        // Initialise serial port.
        let mut port = serialport::new(port, BAUD_RATE)
            .stop_bits(StopBits::One)
            .parity(Parity::None)
            .flow_control(FlowControl::None)
            .data_bits(DataBits::Eight)
            .timeout(Duration::from_millis(1000))
            .open()?;

        let (tx, rx) = mpsc::channel::<Vec<u8>>();

        thread::spawn(move || {
            let mut state = State::Rx;
            let mut buf = Vec::with_capacity(1);

            loop {
                match state {
                    State::Tx => match rx.try_recv() {
                        Ok(bytes) => {
                            println!("write {} bytes", bytes.len());

                            match port.write(&bytes) {
                                Ok(_written) => {
                                    // TODO
                                }
                                Err(_err) => {
                                    // TODO
                                }
                            }
                        }
                        Err(mpsc::TryRecvError::Empty) => {
                            state = State::Rx;
                        }
                        Err(mpsc::TryRecvError::Disconnected) => break,
                    },
                    State::Rx => {
                        if port.bytes_to_read().unwrap() == 0 {
                            state = State::Tx;
                            println!("EOF =================== EOF");
                            continue;
                        }

                        if let Err(_err) = port.read_exact(&mut buf) {
                            break;
                        }

                        println!("{:#04x}", buf[0]);
                    }
                }
            }
        });

        // Initialise radio.
        let rnode = Self { config, tx };
        rnode.set_frequency()?;
        rnode.set_bandwidth()?;
        rnode.set_tx_power()?;
        rnode.set_spreading_factor()?;
        rnode.set_coding_rate()?;
        rnode.set_radio_state(RadioState::ON)?;

        Ok(rnode)
    }

    pub fn send(&self, data: impl Into<Vec<u8>>) -> Result<(), Error> {
        let data = data.into();

        if data.len() > MTU {
            return Err(Error::payload_too_large());
        }

        self.tx.send(data)?;

        Ok(())
    }

    pub fn set_frequency(&self) -> Result<(), Error> {
        let mut command = Vec::new();
        command.extend_from_slice(&[KISS::FEND as u8, RNODE::CMD_FREQUENCY as u8]);
        command.extend_from_slice(&self.config.frequency.to_be_bytes());
        command.extend_from_slice(&[KISS::FEND as u8]);

        self.send(command)?;

        Ok(())
    }

    pub fn set_bandwidth(&self) -> Result<(), Error> {
        let mut command = Vec::new();
        command.extend_from_slice(&[KISS::FEND as u8, RNODE::CMD_BANDWIDTH as u8]);
        command.extend_from_slice(&self.config.bandwidth.to_be_bytes());
        command.extend_from_slice(&[KISS::FEND as u8]);

        self.send(command)?;
        Ok(())
    }

    pub fn set_spreading_factor(&self) -> Result<(), Error> {
        self.send(
            [
                KISS::FEND as u8,
                RNODE::CMD_SF as u8,
                self.config.sf,
                KISS::FEND as u8,
            ]
            .to_vec(),
        )?;
        Ok(())
    }

    pub fn set_tx_power(&self) -> Result<(), Error> {
        self.send(
            [
                KISS::FEND as u8,
                RNODE::CMD_TXPOWER as u8,
                self.config.tx_power,
                KISS::FEND as u8,
            ]
            .to_vec(),
        )?;
        Ok(())
    }

    pub fn set_coding_rate(&self) -> Result<(), Error> {
        self.send(
            [
                KISS::FEND as u8,
                RNODE::CMD_CR as u8,
                self.config.cr,
                KISS::FEND as u8,
            ]
            .to_vec(),
        )?;
        Ok(())
    }

    fn set_radio_state(&self, state: RadioState) -> Result<(), Error> {
        self.send(
            [
                KISS::FEND as u8,
                RNODE::CMD_RADIO_STATE as u8,
                state as u8,
                KISS::FEND as u8,
            ]
            .to_vec(),
        )?;
        Ok(())
    }
}
