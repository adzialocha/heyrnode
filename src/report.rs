use std::sync::{Arc, Mutex};

use crate::config::RadioConfig;

#[derive(Clone, Debug)]
pub(crate) struct Report(Arc<Mutex<Inner>>);

#[derive(Clone, Debug, Default)]
struct Inner {
    frequency: Option<u32>,
    bandwidth: Option<u32>,
    sf: Option<u8>,
    cr: Option<u8>,
    tx_power: Option<u8>,
    radio_state: bool,
    radio_lock: bool,
    stats: Stats,
    random: u8,
}

#[derive(Clone, Debug, Default)]
pub struct Stats {
    /// Data packets sent.
    tx: u32,

    /// Data packets received.
    rx: u32,

    /// Bytes sent.
    tx_bytes: u32,

    /// Bytes received.
    rx_bytes: u32,

    /// Received signal strength indicator (RSSI).
    ///
    /// Measurement of the power present in a received radio signal.
    rssi: u8,

    /// Signal-to-noise ratio (SNR).
    ///
    /// Ratio between the received power signal and the noise floor power level. If SNR is greater
    /// than 0, the received signal operates above the noise floor.
    ///
    /// Typical LoRa SNR values are between -20dB and +10dB.
    snr: u8,

    /// Airtime.
    ats: u16,

    /// Longterm airtime.
    atl: u16,

    /// Total channel utilization.
    cls: u16,

    /// Longterm channel utilization.
    cll: u16,

    /// Current RSSI (+ RSSI offset of 157).
    crs: u8,

    /// Noise floor (+ RSSI offset of 157).
    nfl: u8,

    /// Set to current RSSI if interference detected, otherwise 0xFF (255).
    ntf: u8,

    /// LoRa symbol time (ms).
    lst: u16,

    /// LoRa symbol rate.
    lsr: u16,

    /// LoRa preamble symbols.
    prs: u16,

    /// LoRa preamble time (ms).
    prt: u16,

    /// CSMA slot (ms).
    cst: u16,

    /// DIFS (ms).
    dft: u16,
}

impl Report {
    const BASE_TWO: usize = 2;

    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Inner::default())))
    }

    pub fn set_frequency(&self, frequency: u32) {
        let mut inner = self.0.lock().unwrap();
        inner.frequency = Some(frequency);
    }

    pub fn set_bandwidth(&self, bandwidth: u32) {
        let mut inner = self.0.lock().unwrap();
        inner.bandwidth = Some(bandwidth);
    }

    pub fn set_spreading_factor(&self, sf: u8) {
        let mut inner = self.0.lock().unwrap();
        inner.sf = Some(sf);
    }

    pub fn set_coding_rate(&self, cr: u8) {
        let mut inner = self.0.lock().unwrap();
        inner.cr = Some(cr);
    }

    pub fn set_tx_power(&self, tx_power: u8) {
        let mut inner = self.0.lock().unwrap();
        inner.tx_power = Some(tx_power);
    }

    pub fn set_radio_state(&self, state: bool) {
        let mut inner = self.0.lock().unwrap();
        inner.radio_state = state;
    }

    pub fn set_radio_lock(&self, state: bool) {
        let mut inner = self.0.lock().unwrap();
        inner.radio_lock = state;
    }

    pub fn inc_stat_rx(&self, bytes: u32) {
        let mut inner = self.0.lock().unwrap();
        inner.stats.rx += 1;
        inner.stats.rx_bytes += bytes;
    }

    pub fn inc_stat_tx(&self, bytes: u32) {
        let mut inner = self.0.lock().unwrap();
        inner.stats.tx += 1;
        inner.stats.tx_bytes += bytes;
    }

    pub fn set_stat_rx(&self, rx: u32) {
        let mut inner = self.0.lock().unwrap();
        inner.stats.rx = rx;
    }

    pub fn set_stat_tx(&self, tx: u32) {
        let mut inner = self.0.lock().unwrap();
        inner.stats.tx = tx;
    }

    pub fn set_stat_rssi(&self, rssi: u8) {
        let mut inner = self.0.lock().unwrap();
        inner.stats.rssi = rssi;
    }

    pub fn set_stat_snr(&self, snr: u8) {
        let mut inner = self.0.lock().unwrap();
        inner.stats.snr = snr;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_stat_chtm(&self, ats: u16, atl: u16, cls: u16, cll: u16, crs: u8, nfl: u8, ntf: u8) {
        let mut inner = self.0.lock().unwrap();
        inner.stats.ats = ats;
        inner.stats.atl = atl;
        inner.stats.cls = cls;
        inner.stats.cll = cll;
        inner.stats.crs = crs;
        inner.stats.nfl = nfl;
        inner.stats.ntf = ntf;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_stat_phyprm(&self, lst: u16, lsr: u16, prs: u16, prt: u16, cst: u16, dft: u16) {
        let mut inner = self.0.lock().unwrap();
        inner.stats.lst = lst;
        inner.stats.lsr = lsr;
        inner.stats.prs = prs;
        inner.stats.prt = prt;
        inner.stats.cst = cst;
        inner.stats.dft = dft;
    }

    pub fn set_random(&self, value: u8) {
        let mut inner = self.0.lock().unwrap();
        inner.random = value;
    }

    pub fn verify(&self, config: &RadioConfig) -> bool {
        let inner = self.0.lock().unwrap();

        if let Some(frequency) = inner.frequency
            && config.frequency != frequency
        {
            return false;
        }

        if let Some(bandwidth) = inner.bandwidth
            && config.bandwidth != bandwidth
        {
            return false;
        }

        if let Some(sf) = inner.sf
            && config.sf != sf
        {
            return false;
        }

        if let Some(cr) = inner.cr
            && config.cr != cr
        {
            return false;
        }

        if let Some(tx_power) = inner.tx_power
            && config.tx_power != tx_power
        {
            return false;
        }

        true
    }

    pub fn bitrate(&self) -> f32 {
        let inner = self.0.lock().unwrap();

        let sf = inner.sf.unwrap_or_default() as f32;
        let cr = inner.cr.unwrap_or_default() as f32;
        let bandwidth = inner.bandwidth.unwrap_or_default() as f32;

        sf * ((4.0 / cr) / (Self::BASE_TWO.pow(sf as u32) as f32 / (bandwidth / 1000_f32)))
            * 1000_f32
    }

    pub fn stats(&self) -> Stats {
        self.0.lock().unwrap().stats.clone()
    }
}
