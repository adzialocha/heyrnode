#[repr(u8)]
#[allow(clippy::upper_case_acronyms)]
pub enum KISS {
    FEND = 0xC0,
    FESC = 0xDB,
    TFEND = 0xDC,
    TFESC = 0xDD,
}

impl KISS {
    pub fn escape(bytes: &[u8]) -> Vec<u8> {
        let bytes = Self::replace(
            bytes,
            &[KISS::FESC as u8],
            &[KISS::FESC as u8, KISS::TFESC as u8],
        );
        Self::replace(
            &bytes,
            &[KISS::FEND as u8],
            &[KISS::FEND as u8, KISS::TFEND as u8],
        )
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
