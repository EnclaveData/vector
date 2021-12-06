use crate::{
    config::{log_schema, DataType, Resource, SourceConfig, SourceContext, SourceDescription},
    event::Event,
    internal_events::SplunkTcpEventReceived,
    sources::{Source, util::TcpSource},
    tls::{TlsConfig, MaybeTlsSettings},
};
use bytes::Bytes;
use codec::BytesDelimitedCodec;
use std::net::{SocketAddr, Ipv4Addr};
use serde::{Deserialize, Serialize};
use crate::sources::splunk_tcp::parser::ParsedSplunkTCPEvent;
use crate::sources::util::SocketListenAddr;
use crate::tcp::TcpKeepaliveConfig;

type Error = std::io::Error;

mod parser;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct SplunkTcpConfig {
    /// Local address on which to listen
    #[serde(default = "default_socket_address")]
    address: SocketAddr,
    tls: Option<TlsConfig>,
    max_length: usize,
    keepalive: Option<TcpKeepaliveConfig>,
    shutdown_timeout_secs: u64,
    host_key: Option<String>,
    receive_buffer_bytes: Option<usize>,
}

inventory::submit! {
    SourceDescription::new::<SplunkTcpConfig>("splunk_tcp")
}

impl_generate_config_from_default!(SplunkTcpConfig);

impl SplunkTcpConfig {
    #[cfg(test)]
    pub fn on(address: SocketAddr) -> Self {
        SplunkTcpConfig {
            address,
            ..Self::default()
        }
    }

    pub fn from_address(address: SocketAddr) -> Self {
        Self {
            address,
            ..Self::default()
        }
    }
}

impl Default for SplunkTcpConfig {
    fn default() -> Self {
        SplunkTcpConfig {
            address: default_socket_address(),
            tls: None,
            max_length: 4096,
            keepalive: None,
            shutdown_timeout_secs: default_shutdown_timeout_secs(),
            host_key: None,
            receive_buffer_bytes: None,
        }
    }
}

fn default_socket_address() -> SocketAddr {
    SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 9997)
}

fn default_shutdown_timeout_secs() -> u64 {
    30
}

#[async_trait::async_trait]
#[typetag::serde(name = "splunk_tcp")]
impl SourceConfig for SplunkTcpConfig {
    async fn build(&self, cx: SourceContext) -> crate::Result<Source> {
        // let source = SplunkTcpSource::new(self.clone());
        let config = SplunkTcpConfig::from_address(self.address.into());
        let tcp = SplunkTcpSource {
            config: config.clone(),
        };
        let tls = MaybeTlsSettings::from_config(&config.tls, true)?;
        tcp.run(
            SocketListenAddr::from(config.address),
            config.keepalive,
            config.shutdown_timeout_secs,
            tls,
            config.receive_buffer_bytes,
            cx.shutdown,
            cx.out,
        )
    }

    fn output_type(&self) -> DataType {
        DataType::Log
    }

    fn source_type(&self) -> &'static str {
        "splunk_tcp"
    }

    fn resources(&self) -> Vec<Resource> {
        vec![Resource::tcp(self.address)]
    }
}

#[derive(Debug, Clone)]
struct SplunkTcpSource {
    pub config: SplunkTcpConfig,
}

impl TcpSource for SplunkTcpSource {
    type Error = std::io::Error;
    type Decoder = BytesDelimitedCodec;

    fn decoder(&self) -> BytesDelimitedCodec {
        BytesDelimitedCodec::new_with_max_length(b'\n', self.config.max_length)
    }

    fn build_event(&self, frame: Bytes, host: Bytes) -> Option<Event> {
        let header = parser::parse_header(&frame);

        let new_event = ParsedSplunkTCPEvent { header };

        let mut event = Event::from(new_event.to_string());

        let log = event.as_mut_log();


        // Add source type
        log.insert(log_schema().source_type_key(), Bytes::from("splunk_tcp"));

//         event.as_mut_log().insert(
//             crate::config::log_schema().source_type_key(),
//             Bytes::from("splunk_tcp"),
//         );

        // let host_key = (self.config.host_key.clone())
        //     .unwrap_or_else(|| crate::config::log_schema().host_key().to_string());
        //
        // event.as_mut_log().insert(host_key, host);

        emit!(SplunkTcpEventReceived {});

        Some(event)
    }
}