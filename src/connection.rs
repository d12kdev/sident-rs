use std::time::Duration;

use log::{debug, info};
use tokio::io::AsyncWriteExt;
use tokio_serial::{SerialPort, SerialPortBuilderExt, SerialStream};

use crate::{
    codec::{consts::STX, SICodec}, errors::{DeserializePacketError, DeserializeRawPacketError, NewConnectionError}, packet::{HostboundPacket, RawPacket, StationboundPacket}, packets::{
        hostbound::{GetSystemValueResponse, SetMsModeResponse},
        stationbound::{BeepIfStationReady, GetSystemValue, SetMsMode},
    }, Baudrate, DataAddressAndLength, MsMode, SystemConfig
};

#[derive(Debug)]
pub struct Connection {
    codec: SICodec,
    port: SerialStream,
    station_sys_config: SystemConfig,
}

impl Connection {
    pub async fn new(port_name: &str) -> Result<Self, NewConnectionError> {
        info!("trying to connect to {}", port_name);
        debug!(
            "opening port {} with baudrate of {}",
            port_name,
            Baudrate::High.actual_baudrate()
        );
        let mut port = tokio_serial::new(port_name, Baudrate::High.actual_baudrate())
            .timeout(Duration::from_secs(10))
            .open_native_async()?;

        #[cfg(unix)]
        port.set_exclusive(false)
            .expect("Unable to set serial port exclusive to false");

        port.set_data_bits(tokio_serial::DataBits::Eight)?;
        port.set_parity(tokio_serial::Parity::None)?;
        port.set_stop_bits(tokio_serial::StopBits::One)?;
        port.set_flow_control(tokio_serial::FlowControl::None)?;
        port.set_baud_rate(Baudrate::High.actual_baudrate())?;

        let codec = SICodec::default();

        let mut conn = Self {
            codec,
            port,
            station_sys_config: SystemConfig::default()
        };

        debug!("try 1 - high baudrate, extended protocol");
        debug!("sending 0xFF and STX");
        conn.port.write_all(&[0xFF]).await?;
        tokio::time::sleep(Duration::from_millis(10)).await;
        conn.port.write_all(&[STX]).await?;

        conn.send_packet(&SetMsMode {
            mode: MsMode::Master,
        })
        .await?;

        match conn.receive_packet::<SetMsModeResponse>().await {
            Ok(_) => {
                debug!("got response at high baudrate");
            }
            _ => {
                debug!("try 2 - low baudrate, extended protocol");
                debug!("setting baudrate to {}", Baudrate::Low.actual_baudrate());
                conn.port.set_baud_rate(Baudrate::Low.actual_baudrate())?;
                debug!("sending SetMsMode");
                conn.send_packet(&SetMsMode {
                    mode: MsMode::Master,
                })
                .await?;

                conn.receive_packet::<SetMsModeResponse>().await?;
                debug!("got response at low baudrate");
            }
        }

        debug!("getting protocol config");
        conn.send_packet(&GetSystemValue {
            addr_len: DataAddressAndLength::SystemConfig,
        }).await?;
        let sysv_response = conn.receive_packet::<GetSystemValueResponse>().await?;
        // TODO: replace the unwrap with error handling
        let sysv = SystemConfig::from_bytes(sysv_response.data.as_slice().try_into().unwrap())?;
        conn.station_sys_config = sysv;

        debug!("sending acoustic feedback");
        conn.send_packet(&BeepIfStationReady { beep_count: 2 }).await?;
        conn.receive_and_ignore_packet().await?;

        info!("connected successfully");
        Ok(conn)
    }

    pub async fn send_packet<P: StationboundPacket>(
        &mut self,
        packet: &P,
    ) -> Result<(), std::io::Error> {
        let serialized = self.codec.serialize_packet(packet);
        debug!("HOST -> STATION: {:?} ({:?})", packet, serialized);
        self.port.write_all(&serialized).await?;
        return Ok(());
    }

    pub async fn receive_raw_packet(&mut self) -> Result<RawPacket, DeserializeRawPacketError> {
        let rp = self
            .codec
            .deserialize_raw_packet_reader(&mut self.port, None).await?;
        debug!("RAW: STATION -> HOST: {:?}", rp);
        return Ok(rp);
    }

    pub async fn receive_packet<P: HostboundPacket>(
        &mut self,
    ) -> Result<P, DeserializePacketError> {
        let p = self.receive_raw_packet().await?.deserialize_packet::<P>()?;
        debug!("STATION -> HOST: {:?}", p);
        return Ok(p);
    }

    pub async fn receive_and_ignore_packet(&mut self) -> Result<(), DeserializeRawPacketError> {
        self.receive_raw_packet().await?;
        debug!("PACKET IGNORED");
        return Ok(());
    }
}
