/// Protocol related things
#[allow(unused)]
#[cfg(not(feature = "protocol"))]
mod protocol;

#[cfg(feature = "protocol")]
pub mod protocol;
