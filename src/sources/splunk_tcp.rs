use crate::{
    config::{log_schema, DataType, Resource, SourceConfig, SourceContext, SourceDescription},
};

use std::net::{SocketAddr, Ipv4Addr};
use crate::tls::{TlsConfig, MaybeTlsSettings};
use serde::{de, Deserialize, Serialize};
use crate::sources::Source;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct SplunkTcpConfig {
    /// Local address on which to listen
    #[serde(default = "default_socket_address")]
    address: SocketAddr,
    tls: Option<TlsConfig>,
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
        let source = SplunkTcpSource::new(Self);

        let tls = MaybeTlsSettings::from_config(&self.tls, true)?;
        let listener = tls.bind(&self.address).await?;
        let shutdown = cx.shutdown;

        Ok(Box::pin(async move {

            Ok(())
        }))
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

struct SplunkTcpSource {}
impl SplunkTcpSource {
    fn new(config: &SplunkTcpConfig) -> Self {
        SplunkTcpSource {}
    }
}