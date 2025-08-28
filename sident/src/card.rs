use crate::{codec::SICodec, errors::DeserializeCardPersonalDataError, extract_fixed};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[derive(Debug, Clone)]
/// Personal data of the runner
///
/// Source: SPORTident.CardPersonalData
pub struct CardPersonalData {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    //pub class: Option<String>,
    pub phone: Option<String>,
    pub city: Option<String>,
    pub club: Option<String>,
    pub country: Option<String>,
    pub birthdate: Option<String>,
    pub email: Option<String>,
    pub gender: Option<String>,
    //pub start_no: Option<String>,
    pub street: Option<String>,
    pub zipcode: Option<String>,
    //pub uid: Option<String>
}

impl CardPersonalData {
    /// Deserializes `CardPersonalData` from SI6 format
    ///
    /// * `data` - Data
    ///
    /// Source: SPORTident.Communication.Communication._cardParseCard6PersonalData
    pub fn deserialize_card_6(data: &[u8; 204]) -> Result<Self, DeserializeCardPersonalDataError> {
        let data = SICodec::replace_printer_charset_bytes(data);

        let surname_bytes = &extract_fixed!(&data, 0x00..0x13);
        let first_name_bytes = &extract_fixed!(&data, 0x14..0x27);
        let country_bytes = &extract_fixed!(&data, 0x28..0x2B);
        let club_bytes = &extract_fixed!(&data, 0x2C..0x4F);
        //let user_id_bytes = &extract_fixed!(&data, 0x50..0x5F);
        let phone_bytes = &extract_fixed!(&data, 0x60..0x6F);
        let mail_bytes = &extract_fixed!(&data, 0x70..0x93);
        let street_bytes = &extract_fixed!(&data, 0x94..0xA7);
        let city_bytes = &extract_fixed!(&data, 0xA8..0xB7);
        let zip_bytes = &extract_fixed!(&data, 0xB8..0xBF);
        let gender_bytes = &extract_fixed!(&data, 0xC0..0xC3);
        let birthdate_bytes = &extract_fixed!(&data, 0xC4..0xCB);

        fn d(buf: &[u8]) -> Result<String, std::string::FromUtf8Error> {
            SICodec::decode_iso_8859_1(buf)
        }

        let surname = d(surname_bytes)?;
        let first_name = d(first_name_bytes)?;
        let country = d(country_bytes)?;
        let club = d(club_bytes)?;
        let phone = d(phone_bytes)?;
        let mail = d(mail_bytes)?;
        let street = d(street_bytes)?;
        let city = d(city_bytes)?;
        let zip = d(zip_bytes)?;
        let gender = d(gender_bytes)?;
        let birthdate = d(birthdate_bytes)?;

        fn t(str: String) -> Option<String> {
            if str.is_empty() {
                return None;
            }
            return Some(str);
        }

        return Ok(Self {
            first_name: t(first_name),
            last_name: t(surname),
            phone: t(phone),
            city: t(city),
            club: t(club),
            country: t(country),
            birthdate: t(birthdate),
            email: t(mail),
            gender: t(gender),
            street: t(street),
            zipcode: t(zip),
        });
    }

    /// Deserializes `CardPersonalData` from SI8+ format
    ///
    /// * `data` - Data
    ///
    /// Source: SPORTident.Communication.Communication._cardParseGenericPersonalData
    pub fn deserialize(data: &[u8]) -> Result<Self, DeserializeCardPersonalDataError> {
        fn nonempty_or_none(s: &str) -> Option<String> {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }

        if data.len() > 104 {
            return Err(DeserializeCardPersonalDataError::DataTooLong);
        }

        // replace the printer charset bytes
        let data = SICodec::replace_printer_charset_bytes(&data);
        // decode bytes to string via iso8859-1
        let mut decoded = SICodec::decode_iso_8859_1(&data)?;

        // null terminator support
        if let Some(pos) = decoded.find('\0') {
            decoded.truncate(pos);
        }

        // max 11 data cells
        let mut semicolon_count = 0;
        let mut limit_index = decoded.len();
        for (i, ch) in decoded.char_indices() {
            if ch == ';' {
                semicolon_count += 1;
                if semicolon_count >= 11 {
                    limit_index = i + 1;
                    break;
                }
            }
        }
        decoded.truncate(limit_index);

        let mut fields: Vec<&str> = decoded.split(';').collect();

        let garbage = SICodec::decode_iso_8859_1(&[0xEE, 0xEE])?;
        let garbage_single = SICodec::decode_iso_8859_1(&[0xEE])?;

        if let Some(f0) = fields.get_mut(0) {
            if f0.contains(&garbage) {
                *f0 = "";
            }
        }

        if fields.len() == 1 && fields[0].trim().is_empty() {
            return Err(DeserializeCardPersonalDataError::RequiredFieldsAreEmpty);
        }

        while fields.len() < 11 {
            fields.push("");
        }

        let mut gender = nonempty_or_none(fields[2]);

        // Skip if gender field is just 0xEE
        if let Some(g) = &gender {
            if g == &garbage_single {
                gender = None;
            }
        }

        Ok(CardPersonalData {
            first_name: nonempty_or_none(&fields[0].trim().to_string()),
            last_name: nonempty_or_none(&fields[1].trim().to_string()),
            gender,
            birthdate: nonempty_or_none(fields[3]),
            club: nonempty_or_none(fields[4]),
            email: nonempty_or_none(fields[5]),
            phone: nonempty_or_none(fields[6]),
            city: nonempty_or_none(fields[7]),
            street: nonempty_or_none(fields[8]),
            zipcode: nonempty_or_none(fields[9]),
            country: nonempty_or_none(fields[10]),
        })
    }
}

/// Source: SPORTident.CardType
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[derive(Debug, PartialEq, Eq)]
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
    ComCardAir,
}

impl CardType {
    /// Tries to get `CardType` from SIID
    ///
    /// * `siid` - SIID
    ///
    /// Source: SPORTident.Common.Helper.GetCardTypeFromSiid(uint)
    pub fn from_siid(siid: u32) -> Option<CardType> {
        match siid {
            1..=65000 | 200001..=265000 | 300001..=365000 | 400001..=465000 | 3000000..=3999999 => {
                Some(Self::Card5)
            }
            500000..=999999 => Some(Self::Card6),
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
            _ => None,
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
            CardType::PCard => write!(f, "pCard"),
            CardType::Card5U => write!(f, "Card 5U"),
            CardType::Card5R => write!(f, "Card 5R"),
            CardType::TCard => write!(f, "T-Card"),
            CardType::FCard => write!(f, "fCard"),
            CardType::ActiveCard => write!(f, "Active Card (SIAC)"),
            CardType::ComCardUp => write!(f, "ComCard Up"),
            CardType::ComCardPro => write!(f, "ComCard Pro"),
            CardType::ComCardAir => write!(f, "ComCard Air"),
        }
    }
}
