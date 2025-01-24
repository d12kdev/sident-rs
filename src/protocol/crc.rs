pub fn calculate_crc(data: &[u8]) -> u16 {
    if data.len() < 2 {
        return 0; // response value is "0" for none or one data byte
    }

    const POLYNOM: u16 = 0x8005;

    let mut tmp = ((data[0] as u16) << 8) | (data[1] as u16);
    if data.len() == 2 {
        return tmp; // response value is CRC for two data bytes
    }

    let mut idx = 2;
    for _ in 0..(data.len() / 2) {
        let mut val = 0u16;
        if idx < data.len() {
            val = (data[idx] as u16) << 8;
            idx += 1;
        }
        if idx < data.len() {
            val |= data[idx] as u16;
            idx += 1;
        }

        for _ in 0..16 {
            if tmp & 0x8000 != 0 {
                tmp <<= 1;
                if val & 0x8000 != 0 {
                    tmp += 1;
                }
                tmp ^= POLYNOM;
            } else {
                tmp <<= 1;
                if val & 0x8000 != 0 {
                    tmp += 1;
                }
            }
            val <<= 1;
        }
    }

    tmp
}

#[cfg(test)]
mod tests {
    use super::calculate_crc;

    #[test]
    fn test_crc() {
        let test_data: Vec<u8> = vec![0x53, 0x00, 0x05, 0x01, 0x0F, 0xB5, 0x00, 0x00, 0x1E, 0x08];
        let crc = calculate_crc(&test_data);
        assert_eq!(format!("{:X}", crc), "2C12");
    }
}
