use std::io::{Read, Write};
use std::sync::{Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};

use crate::config::{BAUD_RATE, MTU, RadioConfig, TIMEOUT};
use crate::error::{Error, ErrorKind, Result};
use crate::kiss::KISS;
use crate::report::{Report, Stats};
use crate::rnode::{RNODE, RadioLock, RadioState};

pub struct RNodeInterface {
    config: RadioConfig,
    report: Report,
    port: Mutex<Box<dyn SerialPort>>,
    rx: mpsc::Receiver<Vec<u8>>,
}

impl RNodeInterface {
    const UNSET_COMMAND: u8 = 0xFF;

    pub fn new(port: &str, config: RadioConfig) -> Result<Self> {
        let report = Report::new();

        // Initialise serial port.
        let port = serialport::new(port, BAUD_RATE)
            .stop_bits(StopBits::One)
            .parity(Parity::None)
            .flow_control(FlowControl::None)
            .data_bits(DataBits::Eight)
            .timeout(TIMEOUT)
            .open()?;

        let (tx, rx) = mpsc::channel::<Vec<u8>>();

        {
            let mut port = port.try_clone()?;
            let report = report.clone();

            thread::spawn::<_, Result<()>>(move || {
                let mut serial_buffer = Vec::with_capacity(1);
                let mut last_read = Instant::now();

                let mut escape = false;
                let mut command = Self::UNSET_COMMAND;
                let mut in_frame = false;

                let mut buffer = Vec::with_capacity(MTU);

                loop {
                    if port.bytes_to_read()? == 0 {
                        if Instant::now().duration_since(last_read) > TIMEOUT && !buffer.is_empty()
                        {
                            escape = false;
                            command = Self::UNSET_COMMAND;
                            in_frame = true;
                            buffer = Vec::new();
                        }

                        thread::sleep(Duration::from_millis(8));
                        continue;
                    }

                    port.read_exact(&mut serial_buffer)?;

                    let mut byte = serial_buffer[0];
                    last_read = Instant::now();

                    if !in_frame && byte == KISS::FEND as u8 {
                        escape = false;
                        command = Self::UNSET_COMMAND;
                        in_frame = true;
                        buffer = Vec::new();
                    } else if in_frame {
                        if buffer.is_empty() && command == Self::UNSET_COMMAND {
                            command = byte;
                        } else if command == RNODE::CMD_DATA as u8
                            || command == RNODE::CMD_FREQUENCY as u8
                            || command == RNODE::CMD_BANDWIDTH as u8
                            || command == RNODE::CMD_STAT_RX as u8
                            || command == RNODE::CMD_STAT_TX as u8
                        {
                            if byte == KISS::FESC as u8 {
                                escape = true;
                            } else {
                                if escape {
                                    if byte == KISS::TFEND as u8 {
                                        byte = KISS::FEND as u8;
                                    } else if byte == KISS::TFESC as u8 {
                                        byte = KISS::FESC as u8;
                                    }

                                    escape = false;
                                }

                                buffer.push(byte);
                            }
                        } else {
                            buffer.push(byte);
                        }
                    } else if in_frame && byte == KISS::FEND as u8 {
                        if command == RNODE::CMD_DATA as u8 {
                            tx.send(buffer)?;
                        } else if command == RNODE::CMD_FREQUENCY as u8 {
                            if let Ok(data) = buffer.try_into() {
                                let frequency = u32::from_be_bytes(data);
                                report.set_frequency(frequency);
                                println!("received CMD_FREQUENCY: {:?}", frequency);
                            }
                        } else if command == RNODE::CMD_BANDWIDTH as u8 {
                            if let Ok(data) = buffer.try_into() {
                                let bandwidth = u32::from_be_bytes(data);
                                report.set_bandwidth(bandwidth);
                                println!("received CMD_BANDWIDTH: {:?}", bandwidth);
                            }
                        } else if command == RNODE::CMD_TXPOWER as u8 {
                            if let Some(data) = buffer.pop() {
                                report.set_tx_power(data);
                                println!("received CMD_TXPOWER: {:?}", data);
                            }
                        } else if command == RNODE::CMD_SF as u8 {
                            if let Some(data) = buffer.pop() {
                                report.set_spreading_factor(data);
                                println!("received CMD_SF: {:?}", data);
                            }
                        } else if command == RNODE::CMD_CR as u8 {
                            if let Some(data) = buffer.pop() {
                                report.set_coding_rate(data);
                                println!("received CMD_CR: {:?}", data);
                            }
                        } else if command == RNODE::CMD_RADIO_STATE as u8 {
                            if let Some(data) = buffer.pop() {
                                report.set_radio_state(RadioState::try_from(data)?);
                                println!("received CMD_RADIO_STATE: {:?}", data);
                            }
                        } else if command == RNODE::CMD_RADIO_LOCK as u8 {
                            if let Some(data) = buffer.pop() {
                                report.set_radio_lock(RadioLock::try_from(data)?);
                                println!("received CMD_RADIO_LOCK: {:?}", data);
                            }
                        } else if command == RNODE::CMD_STAT_RX as u8 {
                            if let Ok(data) = buffer.try_into() {
                                let stat_rx = u32::from_be_bytes(data);
                                report.set_stat_rx(stat_rx);
                                println!("received CMD_STAT_RX: {:?}", stat_rx);
                            }
                        } else if command == RNODE::CMD_STAT_TX as u8 {
                            if let Ok(data) = buffer.try_into() {
                                let stat_tx = u32::from_be_bytes(data);
                                report.set_stat_tx(stat_tx);
                                println!("received CMD_STAT_TX: {:?}", stat_tx);
                            }
                        } else if command == RNODE::CMD_STAT_RSSI as u8 {
                            if let Some(data) = buffer.pop() {
                                report.set_stat_rssi(data);
                                println!("received CMD_STAT_RSSI: {:?}", data);
                            }
                        } else if command == RNODE::CMD_STAT_SNR as u8 {
                            if let Some(data) = buffer.pop() {
                                report.set_stat_snr(data);
                                println!("received CMD_STAT_SNR: {:?}", data);
                            }
                        } else if command == RNODE::CMD_RANDOM as u8 {
                            if let Some(data) = buffer.pop() {
                                report.set_random(data);
                                println!("received CMD_RANDOM: {:?}", data);
                            }
                        } else if command == RNODE::CMD_ERROR as u8 {
                            println!("received CMD_ERROR: {:?}", buffer);
                        } else if command == RNODE::CMD_READY as u8 {
                            println!("received CMD_READY: {:?}", buffer);
                        } else {
                            println!("received unknown command: {:?}", buffer);
                        }

                        escape = false;
                        command = Self::UNSET_COMMAND;
                        in_frame = true;
                        buffer = Vec::new();
                    }
                }
            });
        }

        // Initialise radio.
        let rnode = Self {
            config,
            report,
            port: Mutex::new(port),
            rx,
        };

        rnode.set_frequency()?;
        rnode.set_bandwidth()?;
        rnode.set_tx_power()?;
        rnode.set_spreading_factor()?;
        rnode.set_coding_rate()?;
        rnode.set_radio_state(RadioState::ON)?;

        Ok(rnode)
    }

    fn send_command(&self, command: impl AsRef<[u8]>) -> Result<()> {
        if command.as_ref().len() > MTU {
            return Err(Error::from_kind(ErrorKind::PayloadTooLarge));
        }

        {
            let mut port = self.port.lock().unwrap();
            port.write_all(command.as_ref())?;
        }

        Ok(())
    }

    pub fn send(&self, data: impl AsRef<[u8]>) -> Result<()> {
        let mut command = Vec::new();
        command.extend_from_slice(&[KISS::FEND as u8, RNODE::CMD_DATA as u8]);
        command.extend_from_slice(&KISS::escape(data.as_ref()));
        command.extend_from_slice(&[KISS::FEND as u8]);
        self.send_command(command)
    }

    pub fn recv(&self) -> Result<Vec<u8>> {
        let data = self.rx.recv()?;
        Ok(data)
    }

    pub fn verify(&self) -> bool {
        self.report.verify(&self.config)
    }

    pub fn stats(&self) -> Stats {
        self.report.stats()
    }

    pub fn bitrate(&self) -> f32 {
        self.report.bitrate()
    }

    fn set_frequency(&self) -> Result<()> {
        let mut command = Vec::new();
        command.extend_from_slice(&[KISS::FEND as u8, RNODE::CMD_FREQUENCY as u8]);
        command.extend_from_slice(&KISS::escape(&self.config.frequency.to_be_bytes()));
        command.extend_from_slice(&[KISS::FEND as u8]);
        self.send_command(command)
    }

    fn set_bandwidth(&self) -> Result<()> {
        let mut command = Vec::new();
        command.extend_from_slice(&[KISS::FEND as u8, RNODE::CMD_BANDWIDTH as u8]);
        command.extend_from_slice(&KISS::escape(&self.config.bandwidth.to_be_bytes()));
        command.extend_from_slice(&[KISS::FEND as u8]);
        self.send_command(command)
    }

    fn set_spreading_factor(&self) -> Result<()> {
        self.send_command([
            KISS::FEND as u8,
            RNODE::CMD_SF as u8,
            self.config.sf,
            KISS::FEND as u8,
        ])
    }

    fn set_tx_power(&self) -> Result<()> {
        self.send_command([
            KISS::FEND as u8,
            RNODE::CMD_TXPOWER as u8,
            self.config.tx_power,
            KISS::FEND as u8,
        ])
    }

    fn set_coding_rate(&self) -> Result<()> {
        self.send_command([
            KISS::FEND as u8,
            RNODE::CMD_CR as u8,
            self.config.cr,
            KISS::FEND as u8,
        ])
    }

    fn set_radio_state(&self, state: RadioState) -> Result<()> {
        self.send_command([
            KISS::FEND as u8,
            RNODE::CMD_RADIO_STATE as u8,
            state as u8,
            KISS::FEND as u8,
        ])
    }

    #[allow(unused)]
    fn set_promiscuous_mode(&self, mode: bool) -> Result<()> {
        self.send_command([
            KISS::FEND as u8,
            RNODE::CMD_PROMISC as u8,
            u8::from(mode),
            KISS::FEND as u8,
        ])
    }
}
