use std::io::{Read, Write};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};

use crate::config::{BAUD_RATE, MTU, RadioConfig};
use crate::error::{Error, ErrorKind};
use crate::kiss::KISS;
use crate::rnode::{RNODE, RadioState};

pub struct RNodeInterface {
    config: RadioConfig,
    port: Mutex<Box<dyn SerialPort>>,
}

impl RNodeInterface {
    pub fn new(port: &str, config: RadioConfig) -> Result<Self, Error> {
        // Initialise serial port.
        let port = serialport::new(port, BAUD_RATE)
            .stop_bits(StopBits::One)
            .parity(Parity::None)
            .flow_control(FlowControl::None)
            .data_bits(DataBits::Eight)
            .timeout(Duration::from_millis(1000))
            .open()?;

        {
            let mut port = port.try_clone()?;

            thread::spawn(move || {
                let mut buf = Vec::with_capacity(1);

                loop {
                    if port.bytes_to_read().unwrap() == 0 {
                        println!("EOF =================== EOF");
                        continue;
                    }

                    if let Err(_err) = port.read_exact(&mut buf) {
                        break;
                    }

                    if buf[0] == KISS::FEND as u8 {}

                    println!("{:#04x}", buf[0]);
                }
            });
        }

        // Initialise radio.
        let rnode = Self {
            config,
            port: Mutex::new(port),
        };

        rnode.set_frequency()?;
        rnode.set_bandwidth()?;
        rnode.set_tx_power()?;
        rnode.set_spreading_factor()?;
        rnode.set_coding_rate()?;
        rnode.set_radio_state(RadioState::ON)?;

        Ok(rnode)
    }

    fn send_command(&self, command: impl AsRef<[u8]>) -> Result<(), Error> {
        if command.as_ref().len() > MTU {
            return Err(Error::from_kind(ErrorKind::PayloadTooLarge));
        }

        {
            let mut port = self
                .port
                .lock()
                .expect("another user of mutex panicked and poisoned their lock");
            port.write_all(command.as_ref())?;
        }

        Ok(())
    }

    pub fn send(&self, data: impl AsRef<[u8]>) -> Result<(), Error> {
        let mut command = Vec::new();
        command.extend_from_slice(&[KISS::FEND as u8, RNODE::CMD_DATA as u8]);
        command.extend_from_slice(data.as_ref());
        command.extend_from_slice(&[KISS::FEND as u8]);
        self.send_command(command)
    }

    pub fn set_frequency(&self) -> Result<(), Error> {
        let mut command = Vec::new();
        command.extend_from_slice(&[KISS::FEND as u8, RNODE::CMD_FREQUENCY as u8]);
        command.extend_from_slice(&self.config.frequency.to_be_bytes());
        command.extend_from_slice(&[KISS::FEND as u8]);
        self.send_command(command)
    }

    pub fn set_bandwidth(&self) -> Result<(), Error> {
        let mut command = Vec::new();
        command.extend_from_slice(&[KISS::FEND as u8, RNODE::CMD_BANDWIDTH as u8]);
        command.extend_from_slice(&self.config.bandwidth.to_be_bytes());
        command.extend_from_slice(&[KISS::FEND as u8]);
        self.send_command(command)
    }

    pub fn set_spreading_factor(&self) -> Result<(), Error> {
        self.send_command([
            KISS::FEND as u8,
            RNODE::CMD_SF as u8,
            self.config.sf,
            KISS::FEND as u8,
        ])
    }

    pub fn set_tx_power(&self) -> Result<(), Error> {
        self.send_command([
            KISS::FEND as u8,
            RNODE::CMD_TXPOWER as u8,
            self.config.tx_power,
            KISS::FEND as u8,
        ])
    }

    pub fn set_coding_rate(&self) -> Result<(), Error> {
        self.send_command([
            KISS::FEND as u8,
            RNODE::CMD_CR as u8,
            self.config.cr,
            KISS::FEND as u8,
        ])
    }

    fn set_radio_state(&self, state: RadioState) -> Result<(), Error> {
        self.send_command([
            KISS::FEND as u8,
            RNODE::CMD_RADIO_STATE as u8,
            state as u8,
            KISS::FEND as u8,
        ])
    }
}
