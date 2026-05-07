use std::sync::{Arc, Mutex};

use crate::config::RadioConfig;
use crate::error::Result;
use crate::rnode::{RadioLock, RadioState};

#[derive(Clone, Debug)]
pub(crate) struct Report(Arc<Mutex<Inner>>);

#[derive(Clone, Debug, Default)]
struct Inner {
    frequency: Option<u32>,
    bandwidth: Option<u32>,
    sf: Option<u8>,
    cr: Option<u8>,
    tx_power: Option<u8>,
    radio_state: RadioState,
    radio_lock: RadioLock,
    stats: Stats,
    random: u8,
}

#[derive(Clone, Debug, Default)]
pub struct Stats {
    rx: u32,
    tx: u32,
    rssi: u8,
    snr: u8,
}

impl Report {
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

    pub fn set_radio_state(&self, state: RadioState) {
        let mut inner = self.0.lock().unwrap();
        inner.radio_state = state;
    }

    pub fn set_radio_lock(&self, state: RadioLock) {
        let mut inner = self.0.lock().unwrap();
        inner.radio_lock = state;
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

    pub(crate) fn set_random(&self, value: u8) {
        let mut inner = self.0.lock().unwrap();
        inner.random = value;
    }

    pub fn verify(&self, config: &RadioConfig) -> Result<()> {
        let inner = self.0.lock().unwrap();

        if let Some(frequency) = inner.frequency
            && config.frequency != frequency {
                // TODO
            }

        if let Some(bandwidth) = inner.bandwidth
            && config.bandwidth != bandwidth {
                // TODO
            }

        if let Some(sf) = inner.sf
            && config.sf != sf {
                // TODO
            }

        if let Some(cr) = inner.cr
            && config.cr != cr {
                // TODO
            }

        if let Some(tx_power) = inner.tx_power
            && config.tx_power != tx_power {
                // TODO
            }

        Ok(())
    }

    pub fn bitrate(&self) -> f32 {
        let inner = self.0.lock().unwrap();

        let sf = inner.sf.unwrap_or_default() as f32;
        let cr = inner.cr.unwrap_or_default() as f32;
        let bandwidth = inner.bandwidth.unwrap_or_default() as f32;

        let base: usize = 2;
        

        sf
            * ((4.0 / cr) / (base.pow(sf as u32) as f32 / (bandwidth / 1000_f32)))
            * 1000_f32
    }

    pub fn stats(&self) -> Stats {
        let inner = self.0.lock().unwrap();
        inner.stats.clone()
    }
}
