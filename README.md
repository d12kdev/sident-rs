![sident.rs](https://github.com/user-attachments/assets/5924a2b2-cfe1-416e-8435-3b77e0e73d48)
Implementing the SPORTident protocol in Rust.
****
**⚠️ Legacy/base protocol is not supported. It's deprecated**
*⚠️ The project is incomplete and unusable right now.

 **Roadmap**
 - [x] Implement Encoder and Decoder
 - [x] beep :)
 - [x] Reading out SI cards (PARTIALLY FINISHED)
 - [ ] Implement Config+ -like functionality


## How to readout newer SI-Cards (8,10,11,SIAC)

**⚠️ IF YOU'RE PLANNING TO READOUT ON ANDROID, PLEASE CHECK THE REQUIREMENTS IN SIACOM'S README. ALSO NOTE THAT THIS PROJECT IS STILL UNFINISHED, AND ANDROID SUPPORT SHOULD BE CONSIDERED A BLUEPRINT. (but works)**

```rust
use sident::{
    codec::SICodecTimeout,
    connection::{Connection, ReadoutPreference},
    packets::hostbound::SICardNewerDetected,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(not(target_os = "android"))]
    let mut conn = Connection::new("COM3").await?; // replace with your desired port

    #[cfg(target_os = "android")]
    let mut conn = Connection::new();

    // wait for the card to be inserted into the reader
    let detected: SICardNewerDetected = conn
        .receive_packet_custom(SICodecTimeout::Infinite, sident::td())
        .await?;

    let readout_result = conn
        .read_out(&ReadoutPreference::all(), detected.siid)
        .await?;
    let general_readout = readout_result.to_general_readout()?;

    println!("Readout result:\n{:?}", general_readout);

    return Ok(());
}
```
