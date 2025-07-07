
/// Source: SPORTident.CardPersonalData
// TODO: maybe make types for the fields
#[derive(Debug, Clone)]
pub struct CardPersonalData {
    pub class: String,
    pub phone: String,
    pub city: String,
    pub club: String,
    pub country: String,
    pub birthdate: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub gender: String,
    pub start_no: String,
    pub street: String,
    pub zipcode: String,
    pub uid: String
}

/// Source: SPORTident.CardType
#[derive(Debug)]
pub enum CardType {
    Card5,
    Card6,
    Card8,
    Card9,
    Card10,
    Card11,
    PCard,
    Card5U,
    Card5R,
    TCard,
    FCard,
    /// aka SIAC
    ActiveCard,
    ComCardUp,
    ComCardPro,
    ComCardAir
}

impl CardType {

    /// Source: SPORTident.Common.Helper.GetCardTypeFromSiid(uint)
    pub fn from_siid(siid: u32) -> Option<CardType> {
        match siid {
            1..=65000 | 200001..=265000 | 300001..=365000 | 400001..=465000 | 3000000..=3999999 => {
                Some(Self::Card5)
            }
            500000..=999999 =>   Some(Self::Card6),
            1000000..=1999999 => Some(Self::Card9),
            2000000..=2799999 => Some(Self::Card8),
            2800000..=2999999 => Some(Self::ComCardUp),
            4000000..=4999999 => Some(Self::PCard),
            5373953..=5438952 => Some(Self::Card5R),
            5570561..=5635560 => Some(Self::Card5U),
            6000000..=6999999 => Some(Self::TCard),
            7000000..=7999999 => Some(Self::Card10),
            8000000..=8999999 | 16777215 => Some(Self::ActiveCard),
            9000000..=9999999 => Some(Self::Card11),
            14000000..=14999999 => Some(Self::FCard),
            _ => None
        }
    }
}

impl std::fmt::Display for CardType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CardType::Card5 => write!(f, "Card 5"),
            CardType::Card6 => write!(f, "Card 6"),
            CardType::Card8 => write!(f, "Card 8"),
            CardType::Card9 => write!(f, "Card 9"),
            CardType::Card10 => write!(f, "Card 10"),
            CardType::Card11 => write!(f, "Card 11"),
            CardType::PCard => write!(f, "P-Card"),
            CardType::Card5U => write!(f, "Card 5U"),
            CardType::Card5R => write!(f, "Card 5R"),
            CardType::TCard => write!(f, "T-Card"),
            CardType::FCard => write!(f, "F-Card"),
            CardType::ActiveCard => write!(f, "Active Card (SIAC)"),
            CardType::ComCardUp => write!(f, "ComCard Up"),
            CardType::ComCardPro => write!(f, "ComCard Pro"),
            CardType::ComCardAir => write!(f, "ComCard Air"),
        }
    }
}