use crate::common::cnv_attributes;
use opentelemetry_proto::tonic::collector::logs::v1::{
    logs_service_server::LogsService, ExportLogsServiceRequest, ExportLogsServiceResponse,
};
use serde::Serialize;
use std::collections::BTreeMap;
use tokio::sync::mpsc;

/// This is created to flatten the log record to make it more compatible with insta for testing
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExportedLog {
    pub trace_id: String,
    pub span_id: String,
    pub observed_time_unix_nano: u64,
    pub severity_number: i32,
    pub severity_text: String,
    pub body: Option<String>,
    pub attributes: BTreeMap<String, String>,
    pub dropped_attributes_count: u32,
    pub flags: u32,
}

impl From<opentelemetry_proto::tonic::logs::v1::LogRecord> for ExportedLog {
    fn from(value: opentelemetry_proto::tonic::logs::v1::LogRecord) -> Self {
        Self {
            trace_id: hex::encode(value.trace_id),
            span_id: hex::encode(value.span_id),
            observed_time_unix_nano: value.observed_time_unix_nano,
            severity_number: value.severity_number,
            severity_text: value.severity_text,
            body: value.body.map(|value| format!("{:?}", value)),
            attributes: cnv_attributes(&value.attributes),
            dropped_attributes_count: value.dropped_attributes_count,
            flags: value.flags,
        }
    }
}

pub(crate) struct FakeLogsService {
    tx: mpsc::Sender<ExportedLog>,
}

impl FakeLogsService {
    pub fn new(tx: mpsc::Sender<ExportedLog>) -> Self {
        Self { tx }
    }
}

#[tonic::async_trait]
impl LogsService for FakeLogsService {
    async fn export(
        &self,
        request: tonic::Request<ExportLogsServiceRequest>,
    ) -> Result<tonic::Response<ExportLogsServiceResponse>, tonic::Status> {
        let sender = self.tx.clone();
        for el in request
            .into_inner()
            .resource_logs
            .into_iter()
            .flat_map(|rl| rl.scope_logs)
            .flat_map(|sl| sl.log_records)
            .map(ExportedLog::from)
        {
            sender
                .send(el)
                .await
                .inspect_err(|e| eprintln!("failed to send to channel: {e}"))
                .map_err(|err| tonic::Status::from_error(Box::new(err)))?;
        }

        Ok(tonic::Response::new(ExportLogsServiceResponse {
            partial_success: None,
        }))
    }
}
