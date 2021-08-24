use super::InternalEvent;
use metrics::counter;
use serde_json::Error;

#[cfg(feature = "sources-splunk_tcpout")]
pub(crate) use self::source::*;

#[derive(Debug)]
pub(crate) struct SplunkTcpEventSent {
    pub byte_size: usize,
}

impl InternalEvent for SplunkTcpEventSent {
    fn emit_metrics(&self) {
        counter!("processed_bytes_total", self.byte_size as u64);
    }
}

#[derive(Debug)]
pub(crate) struct SplunkTcpEventEncodeError {
    pub error: Error,
}

impl InternalEvent for SplunkTcpEventEncodeError {
    fn emit_logs(&self) {
        error!(
            message = "Error encoding Splunk TCP event to JSON.",
            error = ?self.error,
            internal_log_rate_secs = 30,
        );
    }

    fn emit_metrics(&self) {
        counter!("encode_errors_total", 1);
    }
}

#[cfg(feature = "sources-splunk_tcpout")]
mod source {
    use super::InternalEvent;
    use crate::sources::splunk_hec::ApiError;
    use metrics::counter;

    #[derive(Debug)]
    pub(crate) struct SplunkTcpEventReceived;

    impl InternalEvent for SplunkTcpEventReceived {
        fn emit_logs(&self) {
            trace!(message = "Received one event.");
        }

        fn emit_metrics(&self) {
            counter!("events_in_total", 1);
        }
    }

    #[derive(Debug)]
    pub(crate) struct SplunkTcpRequestReceived<'a> {
        pub path: &'a str,
    }

    impl<'a> InternalEvent for SplunkTcpRequestReceived<'a> {
        fn emit_logs(&self) {
            debug!(
                message = "Received one request.",
                path = %self.path,
                internal_log_rate_secs = 10
            );
        }

        fn emit_metrics(&self) {
            counter!("requests_received_total", 1);
        }
    }

    #[derive(Debug)]
    pub(crate) struct SplunkTcpRequestBodyInvalid {
        pub error: std::io::Error,
    }

    impl InternalEvent for SplunkTcpRequestBodyInvalid {
        fn emit_logs(&self) {
            error!(
                message = "Invalid request body.",
                error = ?self.error,
                internal_log_rate_secs = 10
            );
        }

        fn emit_metrics(&self) {}
    }

    #[derive(Debug)]
    pub(crate) struct SplunkTcpRequestError {
        pub(crate) error: ApiError,
    }

    impl InternalEvent for SplunkTcpRequestError {
        fn emit_logs(&self) {
            error!(
                message = "Error processing request.",
                error = ?self.error,
                internal_log_rate_secs = 10
            );
        }

        fn emit_metrics(&self) {
            counter!("request_errors_total", 1);
        }
    }
}
