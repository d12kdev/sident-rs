use crate::define_data_address_and_length;



define_data_address_and_length! {
    DataAddressAndLength {
        ProtocolConfig => (0x74, 0x01),
        SerialNumber   => (0x00, 0x04),
        FirmwareVersion => (0x05, 0x03),
        ProductModel => (0x0B, 0x02),
        SystemConfig => (0x00, 128),
        SrrConfig => (0x04, 0x01),
        ProducedDate => (0x08, 0x03),
        MemoryKb => (0x0D, 0x01),
        LastModification => (0x75, 0x03)
    }
}
