use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serialport::{DataBits, FlowControl, Parity, StopBits};

use crate::config::{BAUD_RATE, MTU, RadioConfig};
use crate::error::{Error, ErrorKind};
use crate::kiss::KISS;
use crate::rnode::{RNODE, RadioState};

pub struct RNodeInterface {
    config: RadioConfig,
    tx: mpsc::Sender<Vec<u8>>,
}

enum State {
    Tx,
    Rx,
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
            return Err(Error::from_kind(ErrorKind::PayloadTooLarge));
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
