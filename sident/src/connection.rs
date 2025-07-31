use std::time::Duration;

pub use impl_shared::*;

use crate::{codec::SICodecTimeout, MsMode, SystemConfig};

#[cfg(target_os = "android")]
type ConnectionStream = siacom::SIAndroidCom;

#[cfg(not(target_os = "android"))]
type ConnectionStream = tokio_serial::SerialStream;

pub struct Connection {
    stream: ConnectionStream,
    ms_mode: MsMode,
    station_sys_config: SystemConfig
}

#[cfg(not(target_os = "android"))]
mod impl_not_android {
    use std::time::Duration;

    use tokio::io::AsyncWriteExt;

    use log::info;
    use tokio_serial::{SerialPort, SerialPortBuilderExt};

    use crate::{
        Baudrate, MsMode, SystemConfig, SystemValueDataAddressAndLength,
        codec::consts::STX,
        connection::Connection,
        errors::NewConnectionError,
        packets::{hostbound::GetSystemValueResponse, stationbound::GetSystemValue},
    };

    impl super::Connection {
        pub async fn new(port_name: &str) -> Result<Self, NewConnectionError> {
            async fn msmmaster(conn: &mut Connection) -> Result<(), NewConnectionError> {
                if !conn.set_ms_mode(MsMode::Master).await? {
                    return Err(NewConnectionError::FailedToSetMsMode);
                }
                return Ok(());
            }

            info!("trying to connect to {}", port_name);
            info!(
                "opening port {} with baudrate of {}",
                port_name,
                Baudrate::High.actual_baudrate()
            );
            let mut port = tokio_serial::new(port_name, Baudrate::High.actual_baudrate())
                .timeout(Duration::from_secs(2))
                .open_native_async()?;

            #[cfg(target_os = "linux")]
            port.set_exclusive(false)?;

            port.set_data_bits(tokio_serial::DataBits::Eight)?;
            port.set_parity(tokio_serial::Parity::None)?;
            port.set_stop_bits(tokio_serial::StopBits::One)?;
            port.set_flow_control(tokio_serial::FlowControl::None)?;
            port.set_baud_rate(Baudrate::High.actual_baudrate())?;

            let mut conn = Connection {
                stream: port,
                ms_mode: MsMode::Master,
                station_sys_config: SystemConfig::default(),
            };

            info!("try 1 - high baudrate, extended protocol");
            info!("sending 0xFF and STX");
            conn.stream.write_all(&[0xFF]).await?;
            tokio::time::sleep(Duration::from_millis(10)).await;
            conn.stream.write_all(&[STX]).await?;

            let msm_result = msmmaster(&mut conn).await;

            if msm_result.is_err() {
                info!(
                    "did not get response at high baudrate: {:?}",
                    msm_result.err().unwrap()
                );
                info!("trying low baudrate fallback");
                conn.stream.set_baud_rate(Baudrate::Low.actual_baudrate())?;
                msmmaster(&mut conn).await?;
                info!("got response at low baudrate");
            } else {
                info!("got response at high baudrate");
            }

            info!("getting protocol config");
            conn.send_packet(&GetSystemValue {
                addr_len: SystemValueDataAddressAndLength::SystemConfig,
            })
            .await?;
            let sysv_response = conn.receive_packet::<GetSystemValueResponse>().await?;
            let sysv = SystemConfig::deserialize(sysv_response.data.as_slice().try_into()?)?;
            conn.station_sys_config = sysv;

            info!("connected successfully");
            Ok(conn)
        }
    }
}

#[cfg(target_os = "android")]
mod impl_android {

    use std::time::Duration;

    use crate::{
        Baudrate, MsMode, SystemConfig, SystemValueDataAddressAndLength,
        codec::consts::STX,
        connection::Connection,
        errors::NewConnectionError,
        packets::{hostbound::GetSystemValueResponse, stationbound::GetSystemValue},
    };
    use log::info;
    use siacom::SIAndroidCom;
    use tokio::io::AsyncWriteExt;

    impl super::Connection {
        pub async fn new() -> Result<Self, NewConnectionError> {
            info!("creating new connection");
            let mut serial = SIAndroidCom::new().await?;
            info!("setting baudrate to high");
            serial
                .set_baud_rate(Baudrate::High.actual_baudrate())
                .await?;
            let mut conn = Connection {
                stream: serial,
                ms_mode: crate::MsMode::Master,
                station_sys_config: SystemConfig::default(),
                timeout_ms: 5000,
            };

            async fn msmmaster(conn: &mut Connection) -> Result<(), NewConnectionError> {
                if !conn.set_ms_mode(MsMode::Master).await? {
                    return Err(NewConnectionError::FailedToSetMsMode);
                }
                return Ok(());
            }

            //info!("flushing");
            //conn.stream.flush().await?;

            info!("trying to handshake with the station");

            info!("try 1 - high baudrate");
            info!("sending 0xFF and STX");
            conn.stream.write_all(&[0xFF]).await?;
            tokio::time::sleep(Duration::from_millis(10)).await;
            conn.stream.write_all(&[STX]).await?;

            let msm_result = msmmaster(&mut conn).await;

            if msm_result.is_err() {
                info!(
                    "did not get response at high baudrate: {:?}",
                    msm_result.err().unwrap()
                );
                info!("trying low baudrate fallback");
                conn.stream
                    .set_baud_rate(Baudrate::Low.actual_baudrate())
                    .await?;
                msmmaster(&mut conn).await?;
                info!("got response at low baudrate");
            } else {
                info!("got response at high baudrate");
            }

            info!("getting protocol config");
            conn.send_packet(&GetSystemValue {
                addr_len: SystemValueDataAddressAndLength::SystemConfig,
            })
            .await?;
            let sysv_response = conn.receive_packet::<GetSystemValueResponse>().await?;
            let sysv = SystemConfig::deserialize(sysv_response.data.as_slice().try_into()?)?;
            conn.station_sys_config = sysv;
            info!("connected successfully");
            return Ok(conn);
        }
    }
}


pub static TIMEOUT_DEFAULT: once_cell::sync::Lazy<SICodecTimeout> = once_cell::sync::Lazy::new(|| {
    SICodecTimeout::Finite(Duration::from_millis(1000))
});


mod impl_shared {

    use log::{info, warn};
    #[allow(unused_imports)]
    use tokio::io::AsyncWriteExt;

    use crate::{
        card::{CardPersonalData, CardType}, carddef::{
            si10::Card10Def, si11::Card11Def, si8::Card8Def, siac::ActiveCardDef, BlockNeededIntention, BlockNeededResult, CardDefinition
        }, codec::{SICodec, SICodecTimeout}, connection::TIMEOUT_DEFAULT, dedup_enum_array, errors::{
            ConnectionOperationError, ReadoutError, ReadoutResultTransformationError,
            ReceivePacketError, ReceiveRawPacketError, SimpleActionError,
        }, generate_readout_fn, packet::{HostboundPacket, RawPacket, StationboundPacket}, packets::{
            hostbound::{GetSICardNewerResponse, SetMsModeResponse},
            stationbound::{BeepIfStationReady, GetSICardNewer, SetMsMode},
        }, punch::Punch, MsMode, SUPPORTED_CARDS
    };

    use super::Connection;

    impl super::Connection {
        pub async fn beep_if_station_ready(
            &mut self,
            beep_count: u8,
        ) -> Result<(), SimpleActionError> {
            self.send_packet(&BeepIfStationReady { beep_count }).await?;
            self.receive_and_ignore_packet().await?;
            return Ok(());
        }

        pub async fn set_ms_mode(
            &mut self,
            mode: MsMode,
        ) -> Result<bool, ConnectionOperationError> {
            info!("changing msmode to {:?}", mode);

            self.send_packet(&SetMsMode { mode }).await?;
            let response: SetMsModeResponse = self.receive_packet().await?;
            self.ms_mode = response.mode;
            if response.mode != mode {
                warn!("Failed to switch to msmode {:?}, returning Ok(false)", mode);
                return Ok(false);
            }

            info!("successfully switched to msmode {:?}", mode);
            return Ok(true);
        }

        pub async fn send_packet<P: StationboundPacket>(
            &mut self,
            packet: &P,
        ) -> Result<(), std::io::Error> {
            let serialized = SICodec::serialize_packet(packet);
            info!("HOST -> STATION: {:?} ({:?})", packet, serialized);
            self.stream.write_all(&serialized).await?;
            return Ok(());
        }

        pub async fn receive_raw_packet_custom(&mut self, stx_timeout: SICodecTimeout, timeout: SICodecTimeout) -> Result<RawPacket, ReceiveRawPacketError> {
            let rp = SICodec::deserialize_raw_packet_reader(
                &mut self.stream,
                stx_timeout,
                timeout
            ).await?;
            info!("RAW: STATION -> HOST: {:?}", rp);
            return Ok(rp);
        }

        pub async fn receive_raw_packet(&mut self) -> Result<RawPacket, ReceiveRawPacketError> {
            return self.receive_raw_packet_custom(*TIMEOUT_DEFAULT, *TIMEOUT_DEFAULT).await;
        }

        pub async fn receive_packet_custom<P: HostboundPacket>(&mut self, stx_timeout: SICodecTimeout, timeout: SICodecTimeout)  -> Result<P, ReceivePacketError> {
            let p = self.receive_raw_packet_custom(stx_timeout, timeout).await?.deserialize_packet::<P>()?;
            info!("STATION -> HOST: {:?}", p);
            return Ok(p);
        }

        pub async fn receive_packet<P: HostboundPacket>(
            &mut self,
        ) -> Result<P, ReceivePacketError> {
            return self.receive_packet_custom(*TIMEOUT_DEFAULT, *TIMEOUT_DEFAULT).await;
        }

        pub async fn receive_and_ignore_packet_custom(&mut self, stx_timeout: SICodecTimeout, timeout: SICodecTimeout) -> Result<(), ReceiveRawPacketError> {
            self.receive_raw_packet_custom(stx_timeout, timeout).await?;
            info!("PACKET IGNORED");
            return Ok(());
        }

        pub async fn receive_and_ignore_packet(&mut self) -> Result<(), ReceiveRawPacketError> {
            return self.receive_and_ignore_packet_custom(*TIMEOUT_DEFAULT, *TIMEOUT_DEFAULT).await;
        }

        generate_readout_fn!(readout_activecard, ActiveCard, ActiveCardDef);
        generate_readout_fn!(readout_card10, Card10, Card10Def);
        generate_readout_fn!(readout_card11, Card11, Card11Def);
        generate_readout_fn!(readout_card8, Card8, Card8Def);

        pub async fn read_out(
            &mut self,
            preferences: &[ReadoutPreference],
            siid: u32,
        ) -> Result<ReadoutResult, ReadoutError> {
            let card_type = CardType::from_siid(siid).ok_or(ReadoutError::CouldNotGetCardType)?;

            let res = match card_type {
                CardType::Card8 => {
                    ReadoutResult::Card8(self.readout_card8(preferences, siid).await?)
                }
                CardType::Card10 => {
                    ReadoutResult::Card10(self.readout_card10(preferences, siid).await?)
                }
                CardType::Card11 => {
                    ReadoutResult::Card11(self.readout_card11(preferences, siid).await?)
                }
                CardType::ActiveCard => {
                    ReadoutResult::ActiveCard(self.readout_activecard(preferences, siid).await?)
                }
                _ => return Err(ReadoutError::CardNotSupported(card_type)),
            };

            return Ok(res);
        }

        pub async fn read_out_generic<T: CardDefinition>(
            &mut self,
            preferences: &[ReadoutPreference],
            siid: u32,
        ) -> Result<T, ReadoutError> {
            let card_type = CardType::from_siid(siid).ok_or(ReadoutError::CouldNotGetCardType)?;

            if !SUPPORTED_CARDS.contains(&card_type) {
                return Err(ReadoutError::CardNotSupported(card_type));
            }

            let preferences = dedup_enum_array!(preferences);

            let mut carddef = T::new_empty();

            async fn satisfy<TX: CardDefinition>(
                carddef: &mut TX,
                intention: BlockNeededIntention,
                conn: &mut Connection,
            ) -> Result<(), ReadoutError> {
                info!("trying to satisfy intention {:?}", intention);
                while carddef.block_needed(&intention) != BlockNeededResult::NoNeed {
                    let block_needed = carddef.block_needed(&intention);
                    let block_needed = match block_needed {
                        BlockNeededResult::Need(x) => x,
                        BlockNeededResult::NoNeed => unreachable!(),
                    };

                    info!("need block {} ({:?})", block_needed, intention);

                    conn.send_packet(&GetSICardNewer {
                        block_number: block_needed,
                    })
                    .await?;
                    let response: GetSICardNewerResponse = conn.receive_packet().await?;
                    info!("feeding carddef with block {}", &response.block_number);
                    carddef.feed_block(response.block_number, &response.data)?;
                }

                return Ok(());
            }

            // SATISFY THE PREFERENCES
            for preference in preferences {
                info!("doing preference {:?}", preference);
                match preference {
                    ReadoutPreference::CardPersonalData => {
                        satisfy(&mut carddef, BlockNeededIntention::CardPersonalData, self).await?
                    }
                    ReadoutPreference::Punches => {
                        satisfy(&mut carddef, BlockNeededIntention::Punches, self).await?
                    },
                    ReadoutPreference::CardExclusives => {
                        satisfy(&mut carddef, BlockNeededIntention::CardExclusives, self).await?
                    }
                }
            }

            info!("preferences statisfied");

            return Ok(carddef);
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum ReadoutPreference {
        CardPersonalData,
        Punches,
        CardExclusives
    }

    impl ReadoutPreference {
        pub fn all() -> [Self; 3] {
            return [ReadoutPreference::CardPersonalData, ReadoutPreference::Punches, ReadoutPreference::CardExclusives];
        }
    }

    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
    #[cfg_attr(feature = "ts-rs", ts(export))]
    #[derive(Debug)]
    pub enum ReadoutResult {
        Card8(Card8Def),
        Card10(Card10Def),
        Card11(Card11Def),
        ActiveCard(ActiveCardDef),
    }

    impl ReadoutResult {
        pub fn to_general_readout(
            &self,
        ) -> Result<GeneralReadout, ReadoutResultTransformationError> {
            let x: GeneralReadout = self.try_into()?;
            return Ok(x);
        }
    }

    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
    #[cfg_attr(feature = "ts-rs", ts(export))]
    #[derive(Debug)]
    pub struct GeneralReadout {
        pub siid: u32,
        pub personal_data: Option<CardPersonalData>,
        pub clear_check: Punch,
        pub start: Option<Punch>,
        pub finish: Option<Punch>,
        pub punches: Vec<Punch>,
    }

    impl TryFrom<ReadoutResult> for GeneralReadout {
        type Error = ReadoutResultTransformationError;

        fn try_from(value: ReadoutResult) -> Result<Self, Self::Error> {
            Self::try_from(&value)
        }
    }

    impl TryFrom<&ReadoutResult> for GeneralReadout {
        type Error = ReadoutResultTransformationError;

        fn try_from(value: &ReadoutResult) -> Result<Self, Self::Error> {
            fn inner<DEF: CardDefinition>(
                def: &DEF,
            ) -> Result<GeneralReadout, ReadoutResultTransformationError> {
                type E = ReadoutResultTransformationError;
                let siid = def.get_siid().ok_or(E::SiidNone)?;
                let personal_data = match def.get_personal_data() {
                    Some(res) => Some(res?),
                    None => None,
                };
                let clear_check = def.get_clear_check().ok_or(E::ClearCheckNone)?;
                let start = match def.get_start() {
                    Some(maybe) => maybe,
                    None => None,
                };
                let finish = match def.get_finish() {
                    Some(maybe) => maybe,
                    None => None,
                };
                let punches = def.get_punches().ok_or(E::PunchesNone)?;

                return Ok(GeneralReadout {
                    siid,
                    personal_data,
                    clear_check,
                    start,
                    finish,
                    punches,
                });
            }

            type X = ReadoutResult;

            return match value {
                X::Card8(def) => inner(def),
                X::Card10(def) | X::Card11(def) => inner(def),
                X::ActiveCard(def) => inner(def),
            };
        }
    }
}
