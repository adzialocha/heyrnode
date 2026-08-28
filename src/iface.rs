use std::io::{Read, Write};
use std::sync::{Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};
use tracing::{debug, error, trace};

use crate::config::{BAUD_RATE, RadioConfig, SPLIT_PACKET_MTU, TIMEOUT};
use crate::error::{Error, ErrorKind, Result};
use crate::kiss::KISS;
use crate::report::{Report, Stats};
use crate::rnode::RNODE;

#[derive(Debug)]
pub struct RNodeInterface {
    config: RadioConfig,
    report: Report,
    port: Mutex<Box<dyn SerialPort>>,
}

impl RNodeInterface {
    const UNSET_COMMAND: u8 = 0xFF;

    pub fn new(port: &str, config: RadioConfig) -> Result<(Self, mpsc::Receiver<Vec<u8>>)> {
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
                let mut serial_buffer = vec![0; 1];
                let mut last_read = Instant::now();

                let mut escape = false;
                let mut command = Self::UNSET_COMMAND;
                let mut in_frame = false;

                let mut buffer = Vec::with_capacity(SPLIT_PACKET_MTU);

                loop {
                    if port.bytes_to_read()? == 0 {
                        if Instant::now().duration_since(last_read) > TIMEOUT && !buffer.is_empty()
                        {
                            escape = false;
                            command = Self::UNSET_COMMAND;
                            in_frame = false;
                            buffer = Vec::new();
                        }

                        thread::sleep(Duration::from_millis(8));
                        continue;
                    }

                    port.read_exact(&mut serial_buffer)?;

                    let mut byte = serial_buffer[0];
                    last_read = Instant::now();

                    if !in_frame && byte == KISS::FEND as u8 {
                        command = Self::UNSET_COMMAND;
                        in_frame = true;
                        buffer = Vec::new();
                    } else if in_frame && byte != KISS::FEND as u8 {
                        if buffer.is_empty() && command == Self::UNSET_COMMAND {
                            command = byte;
                        } else if command == RNODE::CMD_DATA as u8
                            || command == RNODE::CMD_FREQUENCY as u8
                            || command == RNODE::CMD_BANDWIDTH as u8
                            || command == RNODE::CMD_STAT_RX as u8
                            || command == RNODE::CMD_STAT_TX as u8
                            || command == RNODE::CMD_STAT_CHTM as u8
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
                            report.inc_stat_rx(buffer.len() as u32);
                            tx.send(buffer)?;
                        } else if command == RNODE::CMD_FREQUENCY as u8 {
                            if let Ok(data) = buffer.try_into() {
                                let frequency = u32::from_be_bytes(data);
                                report.set_frequency(frequency);
                                debug!("received CMD_FREQUENCY: {}", frequency);
                            }
                        } else if command == RNODE::CMD_BANDWIDTH as u8 {
                            if let Ok(data) = buffer.try_into() {
                                let bandwidth = u32::from_be_bytes(data);
                                report.set_bandwidth(bandwidth);
                                debug!("received CMD_BANDWIDTH: {}", bandwidth);
                            }
                        } else if command == RNODE::CMD_TXPOWER as u8 {
                            if let Some(data) = buffer.pop() {
                                report.set_tx_power(data);
                                debug!("received CMD_TXPOWER: {}", data);
                            }
                        } else if command == RNODE::CMD_SF as u8 {
                            if let Some(data) = buffer.pop() {
                                report.set_spreading_factor(data);
                                debug!("received CMD_SF: {}", data);
                            }
                        } else if command == RNODE::CMD_CR as u8 {
                            if let Some(data) = buffer.pop() {
                                report.set_coding_rate(data);
                                debug!("received CMD_CR: {}", data);
                            }
                        } else if command == RNODE::CMD_RADIO_STATE as u8 {
                            if let Some(data) = buffer.pop() {
                                let state = data != 0x00;
                                report.set_radio_state(state);
                                debug!("received CMD_RADIO_STATE: {}", state);
                            }
                        } else if command == RNODE::CMD_RADIO_LOCK as u8 {
                            if let Some(data) = buffer.pop() {
                                let state = data != 0x00;
                                report.set_radio_lock(state);
                                debug!("received CMD_RADIO_LOCK: {}", state);
                            }
                        } else if command == RNODE::CMD_STAT_RX as u8 {
                            if let Ok(data) = buffer.try_into() {
                                let stat_rx = u32::from_be_bytes(data);
                                report.set_stat_rx(stat_rx);
                                debug!("received CMD_STAT_RX: {}", stat_rx);
                            }
                        } else if command == RNODE::CMD_STAT_TX as u8 {
                            if let Ok(data) = buffer.try_into() {
                                let stat_tx = u32::from_be_bytes(data);
                                report.set_stat_tx(stat_tx);
                                debug!("received CMD_STAT_TX: {}", stat_tx);
                            }
                        } else if command == RNODE::CMD_STAT_RSSI as u8 {
                            if let Some(data) = buffer.pop() {
                                report.set_stat_rssi(data);
                                debug!("received CMD_STAT_RSSI: {}", data);
                            }
                        } else if command == RNODE::CMD_STAT_SNR as u8 {
                            if let Some(data) = buffer.pop() {
                                report.set_stat_snr(data);
                                debug!("received CMD_STAT_SNR: {}", data);
                            }
                        } else if command == RNODE::CMD_STAT_CHTM as u8 {
                            if let Ok(data) = TryInto::<[u8; 11]>::try_into(buffer) {
                                let ats = u16::from_be_bytes([data[0], data[1]]);
                                let atl = u16::from_be_bytes([data[2], data[3]]);
                                let cls = u16::from_be_bytes([data[4], data[5]]);
                                let cll = u16::from_be_bytes([data[6], data[7]]);
                                let crs = data[8];
                                let nfl = data[9];
                                let ntf = data[10];
                                report.set_stat_chtm(ats, atl, cls, cll, crs, nfl, ntf);
                                debug!("received CMD_STAT_CHTM: {:?}", data);
                            }
                        } else if command == RNODE::CMD_STAT_PHYPRM as u8 {
                            if let Ok(data) = TryInto::<[u8; 12]>::try_into(buffer) {
                                let lst = u16::from_be_bytes([data[0], data[1]]);
                                let lsr = u16::from_be_bytes([data[2], data[3]]);
                                let prs = u16::from_be_bytes([data[4], data[5]]);
                                let prt = u16::from_be_bytes([data[6], data[7]]);
                                let cst = u16::from_be_bytes([data[8], data[9]]);
                                let dft = u16::from_be_bytes([data[10], data[11]]);
                                report.set_stat_phyprm(lst, lsr, prs, prt, cst, dft);
                                debug!("received CMD_STAT_PHYPRM: {:?}", data);
                            }
                        } else if command == RNODE::CMD_STAT_BAT as u8 {
                            if let Ok(data) = TryInto::<[u8; 2]>::try_into(buffer) {
                                // 0x00 = unknown
                                // 0x01 = discharging
                                // 0x02 = charging
                                // 0x03 = charged
                                let state = data[0];
                                let level = data[1];
                                debug!("received CMD_STAT_BAT: {} {}", state, level);
                            }
                        } else if command == RNODE::CMD_STAT_TEMP as u8 {
                            if let Some(data) = buffer.pop() {
                                debug!("received CMD_STAT_TEMP: {}", data);
                            }
                        } else if command == RNODE::CMD_RANDOM as u8 {
                            if let Some(data) = buffer.pop() {
                                report.set_random(data);
                                debug!("received CMD_RANDOM: {}", data);
                            }
                        } else if command == RNODE::CMD_ERROR as u8 {
                            if let Some(error_code) = buffer.pop() {
                                error!("received CMD_ERROR: {}", error_code);
                            }
                        } else if command == RNODE::CMD_READY as u8 {
                            if let Some(data) = buffer.pop() {
                                let state = data != 0x00;
                                debug!("received CMD_READY: {}", state);
                            }
                        } else {
                            trace!(
                                "received unknown command: {} (0x{:x?}) = {:?}",
                                command, command, buffer
                            );
                        }

                        in_frame = false;
                        buffer = Vec::new();
                        command = Self::UNSET_COMMAND;
                    }
                }
            });
        }

        // Initialise radio.
        let rnode = Self {
            config,
            report,
            port: Mutex::new(port),
        };

        std::thread::sleep(std::time::Duration::from_secs(2));

        rnode.set_frequency()?;
        rnode.set_bandwidth()?;
        rnode.set_tx_power()?;
        rnode.set_spreading_factor()?;
        rnode.set_coding_rate()?;
        rnode.set_radio_state(true)?;

        Ok((rnode, rx))
    }

    fn send_command(&self, command: impl AsRef<[u8]>) -> Result<()> {
        let command = command.as_ref();
        debug!("send command: {:?}", command);
        let mut port = self.port.lock().unwrap();
        port.write_all(command)?;
        Ok(())
    }

    pub fn send(&self, data: impl AsRef<[u8]>) -> Result<()> {
        let data = data.as_ref();

        if data.len() > SPLIT_PACKET_MTU {
            return Err(Error::from_kind(ErrorKind::PayloadTooLarge));
        }

        let mut command = Vec::new();
        command.extend_from_slice(&[KISS::FEND as u8, RNODE::CMD_DATA as u8]);
        command.extend_from_slice(&KISS::escape(data));
        command.extend_from_slice(&[KISS::FEND as u8]);

        self.report.inc_stat_tx(data.len() as u32);

        self.send_command(command)
    }

    pub fn verify(&self) -> bool {
        self.report.verify(&self.config)
    }

    pub fn stats(&self) -> Stats {
        // Try to get latest stats from device.
        let _ = self.get_channel_stats();
        let _ = self.get_phy_stats();

        std::thread::sleep(std::time::Duration::from_millis(150));

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

    fn set_radio_state(&self, state: bool) -> Result<()> {
        self.send_command([
            KISS::FEND as u8,
            RNODE::CMD_RADIO_STATE as u8,
            if state { 0x01 } else { 0x00 },
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

    fn get_channel_stats(&self) -> Result<()> {
        self.send_command([
            KISS::FEND as u8,
            RNODE::CMD_STAT_CHTM as u8,
            0xFF,
            KISS::FEND as u8,
        ])
    }

    fn get_phy_stats(&self) -> Result<()> {
        self.send_command([
            KISS::FEND as u8,
            RNODE::CMD_STAT_PHYPRM as u8,
            0xFF,
            KISS::FEND as u8,
        ])
    }
}
