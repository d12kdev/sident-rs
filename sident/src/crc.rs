/// Calculates unsigned 16-bit CRC.
pub fn crc16(data: &[u8]) -> u16 {
    const POLY: u16 = 0x8005;
    const BITF: u16 = 0x8000;

    if data.len() < 2 {
        return 0;
    }

    let mut ptr = 0;
    let mut tmp = u16::from_be_bytes([data[ptr], data[ptr + 1]]);
    ptr += 2;

    if data.len() > 2 {
        let mut i = data.len() / 2;

        while i > 0 {
            let val = if i > 1 {
                if ptr + 1 < data.len() {
                    u16::from_be_bytes([data[ptr], data[ptr + 1]])
                } else {
                    break;
                }
            } else if data.len() % 2 == 1 {
                (data[data.len() - 1] as u16) << 8
            } else {
                0
            };

            ptr += 2;

            let mut val = val;
            for _ in 0..16 {
                if (tmp & BITF) != 0 {
                    tmp <<= 1;
                    if (val & BITF) != 0 {
                        tmp += 1;
                    }
                    tmp ^= POLY;
                } else {
                    tmp <<= 1;
                    if (val & BITF) != 0 {
                        tmp += 1;
                    }
                }
                val <<= 1;
            }

            i -= 1;
        }
    }

    tmp & 0xFFFF
}

/// Calculates unsigned 8-bit CRC.
pub fn crc8(data: &[u8]) -> u8 {
    const POLY: u8 = 0x39;

    let mut crc = 0u8;

    for &byte in data {
        crc ^= byte;
        for _ in 0..8 {
            if crc & 0x80 != 0 {
                crc = (crc << 1) ^ POLY;
            } else {
                crc <<= 1;
            }

            crc &= 0xFF;
        }
    }

    crc
}
