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
