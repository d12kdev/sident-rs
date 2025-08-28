// use crate::define_data_address_and_length;

// define_data_address_and_length! {
//     SystemValueDataAddressAndLength {
//         ProtocolConfig => (0x74, 0x01),
//         SerialNumber   => (0x00, 0x04),
//         FirmwareVersion => (0x05, 0x03),
//         ProductModel => (0x0B, 0x02),
//         SystemConfig => (0x00, 128),
//         SrrConfig => (0x04, 0x01),
//         ProducedDate => (0x08, 0x03),
//         MemoryKb => (0x0D, 0x01),
//         LastModification => (0x75, 0x03)
//     }
// }

use std::ops::Index;

pub mod presets {
    use super::AddrLen;

    pub struct SystemConfigAddrLen;

    impl SystemConfigAddrLen {
        pub fn protocol_config() -> AddrLen {
            return AddrLen::new(0x74, 0x01);
        }

        pub fn serial_number() -> AddrLen {
            return AddrLen::new(0x00, 0x04);
        }

        pub fn firmware_version() -> AddrLen {
            return AddrLen::new(0x05, 0x03);
        }

        pub fn product_model() -> AddrLen {
            return AddrLen::new(0x0B, 0x02);
        }

        pub fn full() -> AddrLen {
            return AddrLen::new(0x00, 128);
        }

        pub fn srr_config() -> AddrLen {
            return AddrLen::new(0x04, 0x01);
        }

        pub fn prod_date() -> AddrLen {
            return AddrLen::new(0x08, 0x03);
        }

        pub fn memory_kb() -> AddrLen {
            return AddrLen::new(0x0D, 0x01);
        }

        pub fn last_modification() -> AddrLen {
            return AddrLen::new(0x75, 0x03);
        }

        pub fn simsrr_serial_number() -> AddrLen {
            return AddrLen::new(0x01, 0x03);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AddrLen {
    address: u8,
    length: u8,
}

impl AddrLen {
    pub fn new(address: u8, length: u8) -> Self {
        Self { address, length }
    }

    pub fn address_byte(&self) -> u8 {
        return self.address;
    }

    pub fn address(&self) -> usize {
        return self.address as usize;
    }

    pub fn offset(&self) -> usize {
        return self.address();
    }

    pub fn length_byte(&self) -> u8 {
        return self.length;
    }

    pub fn length(&self) -> usize {
        return self.length as usize;
    }
}

impl<T> Index<AddrLen> for [T] {
    type Output = [T];

    fn index(&self, index: AddrLen) -> &Self::Output {
        let start = index.address();
        let end = start + index.length();
        &self[start..end]
    }
}
