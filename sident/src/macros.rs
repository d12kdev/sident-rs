#[macro_export]
macro_rules! check_vec_len {
    ($vec:expr, $expected_len:expr, $err_variant:expr) => {
        if $vec.len() != $expected_len as usize {
            return Err($err_variant);
        }
    };
}

#[macro_export]
macro_rules! define_data_address_and_length {
    (
        $enum_name:ident {
            $(
                $variant:ident => ($offset:expr, $length:expr)
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $enum_name {
            $(
                $variant,
            )*
            Custom {
                offset: u8,
                length: u8,
            }
        }

        impl $enum_name {
            pub fn address(&self) -> u8 {
                match self {
                    $(
                        Self::$variant => $offset,
                    )*
                    Self::Custom { offset, .. } => *offset,
                }
            }

            #[inline]
            pub fn offset(&self) -> u8 {
                self.address()
            }

            pub fn length(&self) -> u8 {
                match self {
                    $(
                        Self::$variant => $length,
                    )*
                    Self::Custom { length, .. } => *length,
                }
            }

            pub fn from_address(addr: u8) -> Self {
                match addr {
                    $(
                        x if x == $offset => Self::$variant,
                    )*
                    other => Self::Custom { offset: other, length: 0 },
                }
            }
        }
    };
}

#[macro_export]
macro_rules! extract_fixed {
    ($data:expr, $range:expr) => {{
        const LEN: usize = $range.end - $range.start + 1;
        let slice = $data.get($range.start..=$range.end).unwrap();
        <[u8; LEN]>::try_from(slice).unwrap()
    }};
}

#[macro_export]
macro_rules! dedup_enum_array {
    ($slice:expr) => {{
        {
            use std::collections::HashSet;
            let mut seen = HashSet::new();
            let mut result = Vec::new();

            for item in $slice.iter() {
                if seen.insert(item) {
                    result.push(item.clone());
                }
            }

            result
        }
    }};
}

#[macro_export]
macro_rules! push_unique {
    ($vec:expr, $val:expr) => {
        if !$vec.contains(&$val) {
            $vec.push($val);
        }
    };
}

#[macro_export]
macro_rules! generate_readout_fn {
    ($func_name:ident, $card_variant:ident, $def_type:ty) => {
        pub async fn $func_name(
            &mut self,
            preferences: &[ReadoutPreference],
            siid: u32,
        ) -> Result<$def_type, ReadoutError> {
            let card_type = CardType::from_siid(siid).ok_or(ReadoutError::CouldNotGetCardType)?;
            if card_type != CardType::$card_variant {
                return Err(ReadoutError::ExpectedButGot(
                    CardType::$card_variant,
                    card_type,
                ));
            }

            self.read_out_generic(preferences, siid).await
        }
    };
}
