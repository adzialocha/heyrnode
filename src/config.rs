use std::time::Duration;

pub const BAUD_RATE: u32 = 115_200;

pub const MTU: usize = 508;

pub const TIMEOUT: Duration = Duration::from_millis(100);

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

#[derive(Clone, Debug)]
pub struct RadioConfig {
    pub frequency: u32,
    pub bandwidth: u32,
    pub sf: u8,
    pub cr: u8,
    pub tx_power: u8,
}

impl RadioConfig {
    pub(crate) fn new() -> Self {
        Self {
            frequency: 0,
            bandwidth: 0,
            sf: 0,
            cr: 0,
            tx_power: 0,
        }
    }

    pub fn from_preset(region: Region, preset: Preset) -> Self {
        Self::new()
            .frequency(region.min_frequency())
            .bandwidth(preset.bandwidth())
            .spread_factor(preset.spread_factor())
            .coding_rate(preset.coding_rate())
            .tx_power(region.max_tx_power())
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
