use crate::common::cnv_attributes;
use opentelemetry_proto::tonic::{
    collector::metrics::v1::{
        metrics_service_server::MetricsService, ExportMetricsServiceRequest,
        ExportMetricsServiceResponse,
    },
    metrics::v1 as otel_metrics,
};
use serde::Serialize;
use std::collections::BTreeMap;
use tokio::sync::mpsc;

pub(crate) struct FakeMetricsService {
    tx: mpsc::Sender<ExportedMetric>,
}

impl FakeMetricsService {
    pub fn new(tx: mpsc::Sender<ExportedMetric>) -> Self {
        Self { tx }
    }
}

#[tonic::async_trait]
impl MetricsService for FakeMetricsService {
    async fn export(
        &self,
        request: tonic::Request<ExportMetricsServiceRequest>,
    ) -> Result<tonic::Response<ExportMetricsServiceResponse>, tonic::Status> {
        let sender = self.tx.clone();
        for el in request
            .into_inner()
            .resource_metrics
            .iter()
            .flat_map(|e| e.scope_metrics.to_vec())
            .map(ExportedMetric::from)
        {
            sender
                .send(el)
                .await
                .inspect_err(|e| eprintln!("failed to send to channel: {e}"))
                .map_err(|err| tonic::Status::from_error(Box::new(err)))?;
        }

        Ok(tonic::Response::new(ExportMetricsServiceResponse {
            partial_success: None,
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExportedMetric {
    pub metrics: Vec<Metric>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Metric {
    pub name: String,
    pub description: String,
    pub unit: String,
    pub data: Option<MetricsData>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum MetricsData {
    Gauge(Gauge),
    Sum(Sum),
    Histogram(Histogram),
    ExponentialHistogram(ExponentialHistogram),
    Summary(Summary),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Gauge {
    pub data_points: Vec<NumberDataPoint>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Sum {
    pub data_points: Vec<NumberDataPoint>,
    pub aggregation_temporality: i32,
    pub is_monotonic: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Histogram {
    pub data_points: Vec<HistogramDataPoint>,
    pub aggregation_temporality: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExponentialHistogram {
    pub data_points: Vec<ExponentialHistogramDataPoint>,
    pub aggregation_temporality: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Summary {
    pub data_points: Vec<SummaryDataPoint>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NumberDataPoint {
    pub attributes: BTreeMap<String, String>,
    pub start_time_unix_nano: u64,
    pub time_unix_nano: u64,
    pub exemplars: Vec<Exemplar>,
    pub flags: u32,
    pub value: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HistogramDataPoint {
    pub attributes: BTreeMap<String, String>,
    pub start_time_unix_nano: u64,
    pub time_unix_nano: u64,
    pub count: u64,
    pub sum: Option<f64>,
    pub bucket_counts: Vec<u64>,
    pub explicit_bounds: Vec<f64>,
    pub flags: u32,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub exemplars: Vec<Exemplar>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExponentialHistogramDataPoint {
    pub attributes: BTreeMap<String, String>,
    pub start_time_unix_nano: u64,
    pub time_unix_nano: u64,
    pub count: u64,
    pub sum: Option<f64>,
    pub scale: i32,
    pub zero_count: u64,
    pub positive: Option<Buckets>,
    pub negative: Option<Buckets>,
    pub flags: u32,
    pub exemplars: Vec<Exemplar>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub zero_threshold: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Buckets {
    pub offset: i32,
    pub bucket_counts: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SummaryDataPoint {
    pub attributes: BTreeMap<String, String>,
    pub start_time_unix_nano: u64,
    pub time_unix_nano: u64,
    pub count: u64,
    pub sum: f64,
    pub quantile_values: Vec<ValueAtQuantile>,
    pub flags: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ValueAtQuantile {
    pub quantile: f64,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Exemplar {
    pub filtered_attributes: BTreeMap<String, String>,
    pub time_unix_nano: u64,
    pub span_id: String,
    pub trace_id: String,
    pub value: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Value {
    AsDouble(f64),
    AsInt(i64),
}

impl From<otel_metrics::ScopeMetrics> for ExportedMetric {
    fn from(value: otel_metrics::ScopeMetrics) -> Self {
        ExportedMetric {
            metrics: value
                .metrics
                .iter()
                .map(|m| Metric {
                    name: m.name.clone(),
                    description: m.description.clone(),
                    unit: m.unit.clone(),
                    data: m.data.clone().map(Into::into),
                })
                .collect(),
        }
    }
}

impl From<otel_metrics::metric::Data> for MetricsData {
    fn from(value: otel_metrics::metric::Data) -> Self {
        match value {
            otel_metrics::metric::Data::Gauge(g) => MetricsData::Gauge(g.into()),
            otel_metrics::metric::Data::Sum(s) => MetricsData::Sum(s.into()),
            otel_metrics::metric::Data::Histogram(h) => MetricsData::Histogram(h.into()),
            otel_metrics::metric::Data::ExponentialHistogram(h) => {
                MetricsData::ExponentialHistogram(h.into())
            }
            otel_metrics::metric::Data::Summary(s) => MetricsData::Summary(s.into()),
        }
    }
}

impl From<otel_metrics::Summary> for Summary {
    fn from(value: otel_metrics::Summary) -> Self {
        Self {
            data_points: value.data_points.iter().map(Into::into).collect(),
        }
    }
}

impl From<otel_metrics::ExponentialHistogram> for ExponentialHistogram {
    fn from(value: otel_metrics::ExponentialHistogram) -> Self {
        Self {
            data_points: value.data_points.iter().map(Into::into).collect(),
            aggregation_temporality: value.aggregation_temporality,
        }
    }
}

impl From<otel_metrics::Histogram> for Histogram {
    fn from(value: otel_metrics::Histogram) -> Self {
        Self {
            data_points: value.data_points.iter().map(Into::into).collect(),
            aggregation_temporality: value.aggregation_temporality,
        }
    }
}

impl From<otel_metrics::Sum> for Sum {
    fn from(value: otel_metrics::Sum) -> Self {
        Self {
            data_points: value.data_points.iter().map(Into::into).collect(),
            aggregation_temporality: value.aggregation_temporality,
            is_monotonic: value.is_monotonic,
        }
    }
}

impl From<otel_metrics::Gauge> for Gauge {
    fn from(value: otel_metrics::Gauge) -> Self {
        Self {
            data_points: value.data_points.iter().map(Into::into).collect(),
        }
    }
}

impl From<&otel_metrics::NumberDataPoint> for NumberDataPoint {
    fn from(value: &otel_metrics::NumberDataPoint) -> Self {
        Self {
            attributes: cnv_attributes(&value.attributes),
            start_time_unix_nano: value.start_time_unix_nano,
            time_unix_nano: value.time_unix_nano,
            exemplars: value.exemplars.iter().map(Into::into).collect(),
            flags: value.flags,
            value: value.value.map(Into::into),
        }
    }
}

impl From<&otel_metrics::Exemplar> for Exemplar {
    fn from(value: &otel_metrics::Exemplar) -> Self {
        Self {
            filtered_attributes: cnv_attributes(&value.filtered_attributes),
            time_unix_nano: value.time_unix_nano,
            span_id: hex::encode(&value.span_id),
            trace_id: hex::encode(&value.trace_id),
            value: value.value.map(Into::into),
        }
    }
}

impl From<&otel_metrics::SummaryDataPoint> for SummaryDataPoint {
    fn from(value: &otel_metrics::SummaryDataPoint) -> Self {
        Self {
            attributes: cnv_attributes(&value.attributes),
            start_time_unix_nano: value.start_time_unix_nano,
            time_unix_nano: value.time_unix_nano,
            count: value.count,
            sum: value.sum,
            quantile_values: value.quantile_values.iter().map(Into::into).collect(),
            flags: value.flags,
        }
    }
}

impl From<&otel_metrics::summary_data_point::ValueAtQuantile> for ValueAtQuantile {
    fn from(value: &otel_metrics::summary_data_point::ValueAtQuantile) -> Self {
        Self {
            quantile: value.quantile,
            value: value.value,
        }
    }
}

impl From<&otel_metrics::HistogramDataPoint> for HistogramDataPoint {
    fn from(value: &otel_metrics::HistogramDataPoint) -> Self {
        Self {
            attributes: cnv_attributes(&value.attributes),
            start_time_unix_nano: value.start_time_unix_nano,
            time_unix_nano: value.time_unix_nano,
            count: value.count,
            sum: value.sum,
            bucket_counts: value.bucket_counts.to_vec(),
            flags: value.flags,
            explicit_bounds: value.explicit_bounds.to_vec(),
            max: value.max,
            min: value.min,
            exemplars: value.exemplars.iter().map(Into::into).collect(),
        }
    }
}

impl From<&otel_metrics::ExponentialHistogramDataPoint> for ExponentialHistogramDataPoint {
    fn from(value: &otel_metrics::ExponentialHistogramDataPoint) -> Self {
        Self {
            attributes: cnv_attributes(&value.attributes),
            start_time_unix_nano: value.start_time_unix_nano,
            time_unix_nano: value.time_unix_nano,
            count: value.count,
            sum: value.sum,
            scale: value.scale,
            zero_count: value.zero_count,
            positive: value.positive.as_ref().map(Into::into),
            negative: value.negative.as_ref().map(Into::into),
            flags: value.flags,
            exemplars: value.exemplars.iter().map(Into::into).collect(),
            max: value.max,
            min: value.min,
            zero_threshold: value.zero_threshold,
        }
    }
}

impl From<&otel_metrics::exponential_histogram_data_point::Buckets> for Buckets {
    fn from(value: &otel_metrics::exponential_histogram_data_point::Buckets) -> Self {
        Self {
            offset: value.offset,
            bucket_counts: value.bucket_counts.to_vec(),
        }
    }
}

impl From<otel_metrics::exemplar::Value> for Value {
    fn from(value: otel_metrics::exemplar::Value) -> Self {
        match value {
            otel_metrics::exemplar::Value::AsDouble(n) => Value::AsDouble(n),
            otel_metrics::exemplar::Value::AsInt(n) => Value::AsInt(n),
        }
    }
}

impl From<otel_metrics::number_data_point::Value> for Value {
    fn from(value: otel_metrics::number_data_point::Value) -> Self {
        match value {
            otel_metrics::number_data_point::Value::AsDouble(n) => Value::AsDouble(n),
            otel_metrics::number_data_point::Value::AsInt(n) => Value::AsInt(n),
        }
    }
}
