![sident.rs](https://github.com/user-attachments/assets/5924a2b2-cfe1-416e-8435-3b77e0e73d48)
Implementing the SPORTident protocol in Rust.
****
**⚠️ Legacy/base protocol is not supported at this moment. Note that it is deprecated, but I may add support for it.**
*⚠️ You can use sident to readout, but many planned features are not implemented yet. Also some cards are not tested.*

 **Roadmap**
 - [x] Implement Encoder and Decoder
 - [x] beep
 - [x] Reading out SI cards (PARTIALLY FINISHED; supports modern cards, but lacks support and testing of older ones)
 - [ ] Implement Config+ -like functionality


## How to readout SI-Cards

**⚠️ IF YOU'RE PLANNING TO READOUT ON ANDROID, PLEASE CHECK THE REQUIREMENTS IN SIACOM'S README. ALSO NOTE THAT THIS PROJECT IS STILL UNFINISHED, AND ANDROID SUPPORT SHOULD BE CONSIDERED A BLUEPRINT. (but works)**

```rust
use sident::{
    codec::SICodecTimeout,
    connection::{Connection, ReadoutPreference}
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(not(target_os = "android"))]
    let mut conn = Connection::new("COM4").await?; // replace with your desired port

    #[cfg(target_os = "android")]
    let mut conn = Connection::new().await?;

    // wait for the card to be inserted into the reader
    let siid = conn.wait_for_card_insert().await?;

    let readout_result = conn
        .read_out(&ReadoutPreference::all(), siid)
        .await?;
    let general_readout = readout_result.to_general_readout()?;

    println!("Readout result:\n{:?}", general_readout);

    return Ok(());
}
```
