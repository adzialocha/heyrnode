#[derive(Clone, Debug)]
#[repr(u8)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms, unused)]
pub enum RNODE {
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
    CMD_STAT_CHTM = 0x25,
    CMD_STAT_PHYPRM = 0x26,
    CMD_STAT_BAT = 0x27,
    CMD_STAT_CSMA = 0x28,
    CMD_STAT_TEMP = 0x29,

    CMD_BLINK = 0x30,
    CMD_RANDOM = 0x40,

    CMD_FW_VERSION = 0x50,
    CMD_ROM_READ = 0x51,

    CMD_ERROR = 0x90,
}

#[derive(Clone, Debug)]
#[repr(u8)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms, unused)]
pub enum RNODE_ERROR {
    INITRADIO = 0x01,
    TXFAILED = 0x02,
    EEPROM_LOCKED = 0x03,
    QUEUE_FULL = 0x04,
    MEMORY_LOW = 0x05,
    MODEM_TIMEOUT = 0x06,
    UNKNOWN = 0xFF,
}
