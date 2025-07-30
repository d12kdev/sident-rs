/// The type of the station
/// Source: SPORTident.Communication.ProductType
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, num_enum::TryFromPrimitive, Default)]
#[repr(u16)]
pub enum ProductModel {
    SimSrr = 0x21,
    Bs8SiMaster = 0x8188,
    Bsf7 = 0x8197,
    Bsm7 = 0x9197,
    Bs7S = 0x9597,
    Bs7P = 0xB197,
    Bsf8 = 0x8198,
    Bsf9 = 0x819E,
    Bsf8Ostarter = 0x8190,
    #[default]
    Bsm8 = 0x9198,
    Bsm9 = 0x919F,
    Bs11LoopAntenna = 0x8D99,
    Bs11Large = 0x9D9A,
    Bs11Small = 0xCD9B,
    SiGsmDn = 0x1B9D,
    SiPointGolf = 0x90F1,
    SiPointGolf2 = 0x9072,
    SiPointSportident = 0x92F1,
}
