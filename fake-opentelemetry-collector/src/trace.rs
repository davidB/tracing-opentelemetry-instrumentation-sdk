//! based on https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-otlp/tests/smoke.rs
use crate::common::cnv_attributes;
use opentelemetry_proto::tonic::collector::trace::v1::{
    trace_service_server::TraceService, ExportTraceServiceRequest, ExportTraceServiceResponse,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::mpsc;
use std::sync::Mutex;

use tracing::debug;

/// opentelemetry_proto::tonic::trace::v1::Span is no compatible with serde::Serialize
/// and to be able to test with insta,... it's needed (Debug is not enough to be able to filter unstable value,...)
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExportedSpan {
    pub trace_id: String,
    pub span_id: String,
    pub trace_state: String,
    pub parent_span_id: String,
    pub name: String,
    pub kind: String, //SpanKind,
    pub start_time_unix_nano: u64,
    pub end_time_unix_nano: u64,
    pub attributes: BTreeMap<String, String>,
    pub dropped_attributes_count: u32,
    pub events: Vec<Event>,
    pub dropped_events_count: u32,
    pub links: Vec<Link>,
    pub dropped_links_count: u32,
    pub status: Option<Status>,
}

impl From<opentelemetry_proto::tonic::trace::v1::Span> for ExportedSpan {
    fn from(value: opentelemetry_proto::tonic::trace::v1::Span) -> Self {
        Self {
            trace_id: hex::encode(&value.trace_id),
            span_id: hex::encode(&value.span_id),
            trace_state: value.trace_state.clone(),
            parent_span_id: hex::encode(&value.parent_span_id),
            name: value.name.clone(),
            kind: value.kind().as_str_name().to_owned(),
            start_time_unix_nano: value.start_time_unix_nano,
            end_time_unix_nano: value.end_time_unix_nano,
            attributes: cnv_attributes(&value.attributes),
            dropped_attributes_count: value.dropped_attributes_count,
            events: value.events.iter().map(Event::from).collect(),
            dropped_events_count: value.dropped_events_count,
            links: value.links.iter().map(Link::from).collect(),
            dropped_links_count: value.dropped_links_count,
            status: value.status.map(Status::from),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Serialize)]
pub struct Status {
    message: String,
    code: String,
}

impl From<opentelemetry_proto::tonic::trace::v1::Status> for Status {
    fn from(value: opentelemetry_proto::tonic::trace::v1::Status) -> Self {
        Self {
            message: value.message.clone(),
            code: value.code().as_str_name().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Link {
    pub trace_id: String,
    pub span_id: String,
    pub trace_state: String,
    pub attributes: BTreeMap<String, String>,
    pub dropped_attributes_count: u32,
}

impl From<&opentelemetry_proto::tonic::trace::v1::span::Link> for Link {
    fn from(value: &opentelemetry_proto::tonic::trace::v1::span::Link) -> Self {
        Self {
            trace_id: hex::encode(&value.trace_id),
            span_id: hex::encode(&value.span_id),
            trace_state: value.trace_state.clone(),
            attributes: cnv_attributes(&value.attributes),
            dropped_attributes_count: value.dropped_attributes_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Event {
    time_unix_nano: u64,
    name: String,
    attributes: BTreeMap<String, String>,
    dropped_attributes_count: u32,
}

impl From<&opentelemetry_proto::tonic::trace::v1::span::Event> for Event {
    fn from(value: &opentelemetry_proto::tonic::trace::v1::span::Event) -> Self {
        Self {
            time_unix_nano: value.time_unix_nano,
            name: value.name.clone(),
            attributes: cnv_attributes(&value.attributes),
            dropped_attributes_count: value.dropped_attributes_count,
        }
    }
}

pub(crate) struct FakeTraceService {
    tx: Mutex<mpsc::SyncSender<ExportedSpan>>,
}

impl FakeTraceService {
    pub fn new(tx: mpsc::SyncSender<ExportedSpan>) -> Self {
        Self { tx: Mutex::new(tx) }
    }
}

#[tonic::async_trait]
impl TraceService for FakeTraceService {
    async fn export(
        &self,
        request: tonic::Request<ExportTraceServiceRequest>,
    ) -> Result<tonic::Response<ExportTraceServiceResponse>, tonic::Status> {
        debug!("Sending request into channel...");
        request
            .into_inner()
            .resource_spans
            .into_iter()
            .flat_map(|rs| rs.scope_spans)
            .flat_map(|ss| ss.spans)
            .map(ExportedSpan::from)
            .for_each(|es| {
                self.tx.lock().unwrap().send(es).expect("Channel full");
            });
        Ok(tonic::Response::new(ExportTraceServiceResponse {
            partial_success: None,
        }))
    }
}
