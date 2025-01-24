/// Protocol related things
#[cfg(not(feature = "protocol"))]
mod protocol;

#[cfg(feature = "protocol")]
pub mod protocol;