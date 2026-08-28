use std::time::Duration;

pub const BAUD_RATE: u32 = 115_200;

/// "Split Packet" framing implemented in RNode combining larger messages into (max.) two LoRa
/// packets. This gives us a larger MTU than the "native" LoRa one.
///
/// ```text
/// 508 bytes MTU = (255 bytes MTU * 2 packets) - (1 byte split packet header * 2 packets)
/// ```
pub const SPLIT_PACKET_MTU: usize = 508;

/// "Native" MTU of a single LoRa packet.
pub const SINGLE_MTU: usize = 255;

pub const TIMEOUT: Duration = Duration::from_millis(100);

#[derive(Default)]
pub enum Region {
    EU433,
    #[default]
    EU868,
    US,
}

impl Region {
    pub fn frequency(&self) -> u32 {
        match self {
            Region::EU433 => 433_000_000,
            Region::EU868 => 868_000_000,
            Region::US => 915_000_000,
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
    ShortSlow,
    ShortFast,
    ShortTurbo,
    MediumSlow,
    MediumFast,
    LongModerate,
    LongSlow,
    #[default]
    LongFast,
    LongTurbo,
}

impl Preset {
    fn bandwidth(&self) -> u32 {
        match self {
            Preset::LongModerate | Preset::LongSlow => 125_000,
            Preset::ShortSlow
            | Preset::ShortFast
            | Preset::MediumSlow
            | Preset::MediumFast
            | Preset::LongFast => 250_000,
            Preset::LongTurbo | Preset::ShortTurbo => 500_000,
        }
    }

    fn coding_rate(&self) -> u8 {
        match self {
            Preset::ShortSlow
            | Preset::ShortFast
            | Preset::ShortTurbo
            | Preset::MediumSlow
            | Preset::MediumFast
            | Preset::LongFast => 5,
            Preset::LongModerate | Preset::LongSlow | Preset::LongTurbo => 8,
        }
    }

    fn spread_factor(&self) -> u8 {
        match self {
            Preset::ShortFast | Preset::ShortTurbo => 7,
            Preset::ShortSlow => 8,
            Preset::MediumFast => 9,
            Preset::MediumSlow => 10,
            Preset::LongFast | Preset::LongTurbo | Preset::LongModerate => 11,
            Preset::LongSlow => 12,
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
    pub split_packet_mode: bool,
}

impl RadioConfig {
    pub(crate) fn new() -> Self {
        Self {
            frequency: 0,
            bandwidth: 0,
            sf: 0,
            cr: 0,
            tx_power: 0,
            split_packet_mode: false,
        }
    }

    pub fn from_preset(region: Region, preset: Preset) -> Self {
        Self::new()
            .frequency(region.frequency())
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

    /// "Split Packet" framing implemented in RNode combining larger messages into (max.) two LoRa
    /// packets. This gives us a larger MTU than the "native" LoRa one (255 bytes).
    ///
    /// ```text
    /// 508 bytes MTU = (255 bytes MTU * 2 packets) - (1 byte split packet header * 2 packets)
    /// ```
    pub fn split_packet_mode(mut self, mode: bool) -> Self {
        self.split_packet_mode = mode;
        self
    }
}

impl Default for RadioConfig {
    fn default() -> Self {
        Self::from_preset(Region::default(), Preset::default())
    }
}
