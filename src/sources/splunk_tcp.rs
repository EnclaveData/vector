use crate::{
    config::{log_schema, DataType, Resource, SourceConfig, SourceContext, SourceDescription},
    event::Event,
    internal_events::SplunkTcpEventReceived,
    sources::{Source, socket::tcp, util::TcpSource},
    tls::{TlsConfig, MaybeTlsSettings},
};
use bytes::Bytes;
use codec::BytesDelimitedCodec;
use std::net::{SocketAddr, Ipv4Addr};
use serde::{de, Deserialize, Serialize};

type Error = std::io::Error;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct SplunkTcpConfig {
    /// Local address on which to listen
    #[serde(default = "default_socket_address")]
    address: SocketAddr,
    tls: Option<TlsConfig>,
    max_length: usize,
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
}

impl Default for SplunkTcpConfig {
    fn default() -> Self {
        SplunkTcpConfig {
            address: default_socket_address(),
            tls: None,
            max_length: 4096,
        }
    }
}

fn default_socket_address() -> SocketAddr {
    SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 9997)
}

#[async_trait::async_trait]
#[typetag::serde(name = "splunk_tcp")]
impl SourceConfig for SplunkTcpConfig {
    async fn build(&self, cx: SourceContext) -> crate::Result<Source> {
        let source = SplunkTcpSource::new(self.clone());
        let config = tcp::TcpConfig::from_address(self.address.into());
        let tcp = tcp::RawTcpSource {
            config: config.clone(),
        };
        let tls = MaybeTlsSettings::from_config(config.tls(), true)?;
        tcp.run(
            config.address(),
            config.keepalive(),
            config.shutdown_timeout_secs(),
            tls,
            config.receive_buffer_bytes(),
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

struct SplunkTcpSource {
    pub config: SplunkTcpConfig,
}
impl SplunkTcpSource {
    fn new(config: SplunkTcpConfig) -> Self {
        SplunkTcpSource { config }
    }

    fn decoder(&self) -> BytesDelimitedCodec {
        BytesDelimitedCodec::new_with_max_length(b'\n', self.config.max_length)
    }

    fn build_event(&self, frame: Bytes, host: Bytes) -> Option<Event> {
        let byte_size = frame.len();
        let mut event = Event::from(frame);

        event.as_mut_log().insert(
            crate::config::log_schema().source_type_key(),
            Bytes::from("splunk_tcp"),
        );

        // let host_key = (self.config.host_key.clone())
        //     .unwrap_or_else(|| crate::config::log_schema().host_key().to_string());
        //
        // event.as_mut_log().insert(host_key, host);

        emit!(SplunkTcpEventReceived {});

        Some(event)
    }
}