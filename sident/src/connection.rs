use std::time::Duration;

use log::{debug, info, warn};
use tokio::io::AsyncWriteExt;

#[cfg(not(target_os = "android"))]
use tokio_serial::{SerialPort, SerialPortBuilderExt};

use crate::{
    Baudrate, MsMode, SUPPORTED_CARDS, SystemConfig,
    addr_len::presets::SystemConfigAddrLen,
    card::{CardPersonalData, CardType},
    carddef::{
        BlockNeededIntention, BlockNeededResult, CardDefinition, comcardpro::ComCardProDef,
        comcardup::ComCardUpDef, si8::Card8Def, si9::Card9Def, si10::Card10Def, si11::Card11Def,
        siac::ActiveCardDef,
    },
    codec::{SICodec, SICodecTimeout, consts::STX},
    dedup_enum_array,
    errors::{
        ConnectionOperationError, DeserializePacketError, NewConnectionError, ReadoutError,
        ReadoutResultTransformationError, ReceivePacketError, ReceiveRawPacketError,
        SimpleActionError,
    },
    generate_readout_fn,
    packet::{HostboundPacket, Packet, RawPacket, StationboundPacket},
    packets::{
        hostbound::{
            GetSICardNewerResponse, GetSystemValueResponse, SICard5Detected, SICard6Detected,
            SICardNewerDetected, SICardRemoved, SetMsModeResponse,
        },
        stationbound::{BeepIfStationReady, GetSICardNewer, GetSystemValue, SetMsMode},
    },
    product::ProductModel,
    punch::Punch,
};

/// `ConnectionStream` differs on other platforms. For this platform it is `siacom::SIAndroidCom`.
///
/// **Note: Android communication is EXPERIMENTAL.**
#[cfg(target_os = "android")]
type ConnectionStream = siacom::SIAndroidCom;

/// `ConnectionStream` differs on other platforms. For this platform it is `tokio_serial::SerialStream`.
#[cfg(not(target_os = "android"))]
type ConnectionStream = tokio_serial::SerialStream;

/// Struct for controlling the communication with the SPORTident station.
pub struct Connection {
    stream: ConnectionStream,
    ms_mode: MsMode,
    system_config: Option<SystemConfig>,
}

pub static TIMEOUT_DEFAULT: once_cell::sync::Lazy<SICodecTimeout> =
    once_cell::sync::Lazy::new(|| SICodecTimeout::Finite(Duration::from_millis(2500)));

impl Connection {
    /// Tries to connect to the port and returns a new connection.
    ///
    /// * `port_name` - Name of the port to connect to (Not on Android)
    ///
    /// # Example
    /// ```
    /// use sident::connection::Connection;
    ///
    /// // Just Connection::new() on Android
    /// let mut conn = Connection::new("COM4")
    ///     .await
    ///     .expect("Failed to create new SI connection");
    /// ```
    pub async fn new(
        #[cfg(not(target_os = "android"))] port_name: &str,
    ) -> Result<Self, NewConnectionError> {
        /// Set the M/S mode to Master.
        ///
        /// If `set_ms_mode` is `Err` or `false` then returns `NewConnectionError::FailedToSetMsMode`
        async fn msmmaster(conn: &mut Connection) -> Result<(), NewConnectionError> {
            if !conn.set_ms_mode(MsMode::Master).await? {
                return Err(NewConnectionError::FailedToSetMsMode);
            }
            return Ok(());
        }

        #[cfg(target_os = "android")]
        info!("trying to connect to SI");

        #[cfg(not(target_os = "android"))]
        {
            info!("trying to connect to {}", port_name);
            debug!(
                "opening port {} with baudrate of {}",
                port_name,
                Baudrate::High.actual_baudrate()
            );
        }

        #[cfg(not(target_os = "android"))]
        let mut port = tokio_serial::new(port_name, Baudrate::High.actual_baudrate())
            .timeout(Duration::from_secs(2))
            .open_native_async()?;

        #[cfg(target_os = "android")]
        let port = siacom::SIAndroidCom::new().await?;

        #[cfg(target_os = "linux")]
        port.set_exclusive(false)?;

        #[cfg(not(target_os = "android"))]
        {
            port.set_data_bits(tokio_serial::DataBits::Eight)?;
            port.set_parity(tokio_serial::Parity::None)?;
            port.set_stop_bits(tokio_serial::StopBits::One)?;
            port.set_flow_control(tokio_serial::FlowControl::None)?;
            port.set_baud_rate(Baudrate::High.actual_baudrate())?;
        }

        let mut conn = Connection {
            stream: port,
            ms_mode: MsMode::Master,
            system_config: None,
        };

        debug!("try 1 - high baudrate, extended protocol");
        debug!("sending 0xFF and STX");
        conn.stream.write_all(&[0xFF]).await?;
        tokio::time::sleep(Duration::from_millis(10)).await;
        conn.stream.write_all(&[STX]).await?;

        let msm_result = msmmaster(&mut conn).await;

        if msm_result.is_err() {
            debug!(
                "did not get response at high baudrate: {:?}",
                msm_result.err().unwrap()
            );
            debug!("trying low baudrate fallback");
            conn.set_stream_baudrate(Baudrate::Low).await?;
            msmmaster(&mut conn).await?;
            debug!("got response at low baudrate");
        } else {
            debug!("got response at high baudrate");
        }

        debug!("getting protocol config");
        conn.send_packet(&GetSystemValue {
            addr_len: SystemConfigAddrLen::full(),
        })
        .await?;
        let sysv_response = conn.receive_packet::<GetSystemValueResponse>().await?;
        let sysv = SystemConfig::deserialize(sysv_response.data.as_slice().try_into()?)?;
        conn.system_config = Some(sysv);

        info!("connected successfully");
        Ok(conn)
    }

    async fn set_stream_baudrate(&mut self, baudrate: Baudrate) -> std::io::Result<()> {
        #[cfg(target_os = "android")]
        return self.stream.set_baudrate(baudrate.actual_baudrate()).await;
        #[cfg(not(target_os = "android"))]
        {
            self.stream.set_baud_rate(baudrate.actual_baudrate())?;
            return Ok(());
        }
    }

    /// Returns the model of the connected device.
    pub fn get_product_model(&self) -> Option<ProductModel> {
        let sys_conf = self.system_config.as_ref()?;
        return Some(sys_conf.model);
    }

    /// Waits for the card to be inserted and returns the SIID
    ///
    /// *This supports all cards*
    pub async fn wait_for_card_insert(&mut self) -> Result<u32, ReceivePacketError> {
        let raw = self
            .receive_raw_packet_custom(SICodecTimeout::Infinite, crate::td())
            .await?;
        if let RawPacket::Body(body) = &raw {
            let siid = match body.id {
                SICardNewerDetected::PACKET_ID => {
                    raw.deserialize_packet::<SICardNewerDetected>()?.siid
                }
                SICard6Detected::PACKET_ID => raw.deserialize_packet::<SICard6Detected>()?.siid,
                SICard5Detected::PACKET_ID => raw.deserialize_packet::<SICard5Detected>()?.siid,
                _ => {
                    return Err(DeserializePacketError::Other(
                        "Received packet is not a card detected packet".into(),
                    )
                    .into());
                }
            };
            return Ok(siid);
        } else {
            return Err(DeserializePacketError::ResponseIsNak.into());
        }
    }

    /// Makes the station beep, if it is ready.
    ///
    /// * `beep_count` - Beep count
    pub async fn beep_if_station_ready(&mut self, beep_count: u8) -> Result<(), SimpleActionError> {
        self.send_packet(&BeepIfStationReady { beep_count }).await?;
        self.receive_and_ignore_packet().await?;
        return Ok(());
    }

    /// Tries to set the M/S mode.
    ///
    /// * `mode` - The desired M/S mode
    ///
    /// Returns `true` if success
    pub async fn set_ms_mode(&mut self, mode: MsMode) -> Result<bool, ConnectionOperationError> {
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

    /// Sends a packet to the station.
    ///
    /// * `packet` - Packet to send
    ///
    /// ```
    /// use sident::packets::stationbound::BeepIfStationReady;
    ///
    /// conn.send_packet(&BeepIfStationReady { beep_count: 2 }).await.unwrap();
    /// ```
    pub async fn send_packet<P: StationboundPacket>(
        &mut self,
        packet: &P,
    ) -> Result<(), std::io::Error> {
        let serialized = SICodec::serialize_packet(packet);
        debug!("HOST -> STATION: {:?} ({:?})", packet, serialized);
        self.stream.write_all(&serialized).await?;
        return Ok(());
    }

    /// Receives a raw packet. With custom timeout.
    ///
    /// * `stx_timeout` - Timeout for the first `STX` byte
    /// * `timeout` - Timeout for reading next byte
    pub async fn receive_raw_packet_custom(
        &mut self,
        stx_timeout: SICodecTimeout,
        timeout: SICodecTimeout,
    ) -> Result<RawPacket, ReceiveRawPacketError> {
        let rp =
            SICodec::deserialize_raw_packet_reader(&mut self.stream, stx_timeout, timeout).await?;
        debug!("RAW: STATION -> HOST: {:?}", rp);
        return Ok(rp);
    }

    /// Receives a raw packet. With default timeout.
    pub async fn receive_raw_packet(&mut self) -> Result<RawPacket, ReceiveRawPacketError> {
        return self
            .receive_raw_packet_custom(*TIMEOUT_DEFAULT, *TIMEOUT_DEFAULT)
            .await;
    }

    /// Receives and tries to parse a packet. With custom timeout.
    ///
    /// * `<P: HostboundPacket>` - Expected packet (generic)
    /// * `stx_timeout` - Timeout for the first `STX` byte
    /// * `timeout` - Timeout for reading next byte
    pub async fn receive_packet_custom<P: HostboundPacket>(
        &mut self,
        stx_timeout: SICodecTimeout,
        timeout: SICodecTimeout,
    ) -> Result<P, ReceivePacketError> {
        let p = self
            .receive_raw_packet_custom(stx_timeout, timeout)
            .await?
            .deserialize_packet::<P>()?;
        debug!("STATION -> HOST: {:?}", p);
        return Ok(p);
    }

    /// Receives and tries to parse a packet. With default timeout.
    ///
    /// * `<P: HostboundPacket>` - Expected packet (generic)
    pub async fn receive_packet<P: HostboundPacket>(&mut self) -> Result<P, ReceivePacketError> {
        return self
            .receive_packet_custom(*TIMEOUT_DEFAULT, *TIMEOUT_DEFAULT)
            .await;
    }

    /// Receives a raw packet and ignores it. With custom timeout.
    ///
    /// * `stx_timeout` - Timeout for the first `STX` byte
    /// * `timeout` - Timeout for reading next byte
    pub async fn receive_and_ignore_packet_custom(
        &mut self,
        stx_timeout: SICodecTimeout,
        timeout: SICodecTimeout,
    ) -> Result<(), ReceiveRawPacketError> {
        self.receive_raw_packet_custom(stx_timeout, timeout).await?;
        debug!("PACKET IGNORED");
        return Ok(());
    }

    /// Receives a raw packet and ignores it. With default timeout.
    pub async fn receive_and_ignore_packet(&mut self) -> Result<(), ReceiveRawPacketError> {
        return self
            .receive_and_ignore_packet_custom(*TIMEOUT_DEFAULT, *TIMEOUT_DEFAULT)
            .await;
    }

    generate_readout_fn!(readout_activecard, ActiveCard, ActiveCardDef);
    generate_readout_fn!(readout_card11, Card11, Card11Def);
    generate_readout_fn!(readout_card10, Card10, Card10Def);
    generate_readout_fn!(readout_card9, Card9, Card9Def);
    generate_readout_fn!(readout_card8, Card8, Card8Def);
    generate_readout_fn!(readout_comcardpro, ComCardPro, ComCardProDef);
    generate_readout_fn!(readout_comcardup, ComCardUp, ComCardUpDef);

    /// Reads out the card.
    ///
    /// **Note: The card series must be one of the supported ones** (see sident::SUPPORTED_CARDS).
    ///
    /// * `preferences` - Readout preferences (see the `ReadoutPreference` enum for more info)
    /// * `siid` - SIID (card id)
    ///
    /// Returns `ReadoutResult` (see `ReadoutResult` enum for more info).
    pub async fn read_out(
        &mut self,
        preferences: &[ReadoutPreference],
        siid: u32,
    ) -> Result<ReadoutResult, ReadoutError> {
        let card_type = CardType::from_siid(siid).ok_or(ReadoutError::CouldNotGetCardType)?;

        let res = match card_type {
            CardType::ActiveCard => {
                ReadoutResult::ActiveCard(self.readout_activecard(preferences, siid).await?)
            }
            CardType::Card11 => {
                ReadoutResult::Card11(self.readout_card11(preferences, siid).await?)
            }
            CardType::Card10 => {
                ReadoutResult::Card10(self.readout_card10(preferences, siid).await?)
            }
            CardType::Card9 => ReadoutResult::Card9(self.readout_card9(preferences, siid).await?),
            CardType::Card8 => ReadoutResult::Card8(self.readout_card8(preferences, siid).await?),
            CardType::ComCardPro => {
                ReadoutResult::ComCardPro(self.readout_comcardpro(preferences, siid).await?)
            }
            CardType::ComCardUp => {
                ReadoutResult::ComCardUp(self.readout_comcardup(preferences, siid).await?)
            }
            _ => return Err(ReadoutError::CardNotSupported(card_type)),
        };

        return Ok(res);
    }

    /// Reads out the card to the specified `CardDefinition`.
    ///
    /// **Note: The card series must be one of the supported ones** (see sident::SUPPORTED_CARDS).
    ///
    /// * `<T: CardDefinition>` - Card definition (generic)
    /// * `preferences` - Readout preferences (see the `ReadoutPreference` enum for more info)
    /// * `siid` - SIID (card id)
    ///
    /// Returns `<T: CardDefinition>`.
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
            debug!("trying to satisfy intention {:?}", intention);
            while carddef.block_needed(&intention) != BlockNeededResult::NoNeed {
                let block_needed = carddef.block_needed(&intention);
                let block_needed = match block_needed {
                    BlockNeededResult::Need(x) => x,
                    BlockNeededResult::NoNeed => unreachable!(),
                };

                debug!("need block {} ({:?})", block_needed, intention);

                conn.send_packet(&GetSICardNewer {
                    block_number: block_needed,
                })
                .await?;

                let raw_response = match conn.receive_raw_packet().await? {
                    RawPacket::Body(ok) => ok,
                    RawPacket::Nak => return Err(ReadoutError::NakResponse),
                };

                let response: GetSICardNewerResponse = match raw_response.id {
                    GetSICardNewerResponse::PACKET_ID => {
                        GetSICardNewerResponse::deserialize(raw_response.data)?
                    }
                    SICardRemoved::PACKET_ID => return Err(ReadoutError::CardRemoved),
                    _ => return Err(ReadoutError::UnexpectedPacket),
                };

                debug!("feeding carddef with block {}", &response.block_number);
                carddef.feed_block(response.block_number, &response.data)?;
            }

            return Ok(());
        }

        // SATISFY THE PREFERENCES
        for preference in preferences {
            debug!("doing preference {:?}", preference);
            match preference {
                ReadoutPreference::CardPersonalData => {
                    satisfy(&mut carddef, BlockNeededIntention::CardPersonalData, self).await?
                }
                ReadoutPreference::Punches => {
                    satisfy(&mut carddef, BlockNeededIntention::Punches, self).await?
                }
                ReadoutPreference::CardExclusives => {
                    satisfy(&mut carddef, BlockNeededIntention::CardExclusives, self).await?
                }
            }
        }

        debug!("preferences satisfied");

        return Ok(carddef);
    }
}

/// `ReadoutPreference` is used to specify which blocks should sident readout to make the process faster.
///
/// Simply - Thanks to `ReadoutPreference` sident reads out only blocks you need.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReadoutPreference {
    CardPersonalData,
    Punches,
    CardExclusives,
}

impl ReadoutPreference {
    /// Returns all `ReadoutPreference`s
    pub fn all() -> [Self; 3] {
        return [
            ReadoutPreference::CardPersonalData,
            ReadoutPreference::Punches,
            ReadoutPreference::CardExclusives,
        ];
    }
}

/// `ReadoutResult` can hold one of the supported cards definition.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
#[derive(Debug)]
pub enum ReadoutResult {
    ActiveCard(ActiveCardDef),
    Card11(Card11Def),
    Card10(Card10Def),
    Card9(Card9Def),
    Card8(Card8Def),
    ComCardPro(ComCardProDef),
    ComCardUp(ComCardUpDef),
}

impl ReadoutResult {
    /// Tries to transform the `ReadoutResult` into `GeneralReadout`.
    pub fn to_general_readout(&self) -> Result<GeneralReadout, ReadoutResultTransformationError> {
        let x: GeneralReadout = self.try_into()?;
        return Ok(x);
    }
}

/// `GeneralReadout` is just the general information from any `CardDefinition`.
///
/// `GeneralReadout` contains:
/// * SIID
/// * Personal Data (optional)
/// * Clear/Check punch
/// * Start punch (optional)
/// * Finish punch (optional)
/// * Punches
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
            X::ActiveCard(def) | X::Card11(def) => inner(def),
            X::Card10(def) | X::ComCardPro(def) => inner(def),
            X::Card9(def) => inner(def),
            X::Card8(def) | X::ComCardUp(def) => inner(def),
        };
    }
}
