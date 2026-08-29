use bytes::{BufMut, Bytes, BytesMut};

/// KISS (Keep It Simple, Stupid) is a protocol for communicating with a serial terminal node
/// controller (TNC) device used for amateur radio.
#[repr(u8)]
#[allow(clippy::upper_case_acronyms)]
pub enum KISS {
    /// Frame End.
    FEND = 0xC0,

    /// Frame Escape.
    FESC = 0xDB,

    /// Transposed Frame End.
    TFEND = 0xDC,

    /// Transposed Frame Escape.
    TFESC = 0xDD,
}

impl KISS {
    /// If the FEND or FESC codes appear in the data to be transferred, they need to be escaped. The
    /// FEND code is then sent as FESC, TFEND and the FESC is then sent as FESC, TFESC.
    pub fn escape(bytes: Bytes) -> Bytes {
        let bytes = Self::replace(
            bytes,
            KISS::FESC as u8,
            &[KISS::FESC as u8, KISS::TFESC as u8],
        );

        Self::replace(
            bytes,
            KISS::FEND as u8,
            &[KISS::FEND as u8, KISS::TFEND as u8],
        )
    }

    fn replace(source: Bytes, from: u8, to: &[u8]) -> Bytes {
        // Count occurrences.
        let mut count = 0;
        for &b in source.iter() {
            if b == from {
                count += 1;
            }
        }
        if count == 0 {
            return source;
        }

        // Pre-allocate exact capacity.
        let extra = to.len().saturating_sub(1) * count;
        let mut out = BytesMut::with_capacity(source.len() + extra);

        for &b in source.iter() {
            if b == from {
                out.extend_from_slice(to);
            } else {
                out.put_u8(b);
            }
        }

        out.freeze()
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::KISS;

    #[test]
    fn escape() {
        assert_eq!(
            KISS::escape(Bytes::from_iter([KISS::FEND as u8, 54, 45, 53, 54])),
            vec![KISS::FEND as u8, KISS::TFEND as u8, 54, 45, 53, 54]
        );
    }
}
