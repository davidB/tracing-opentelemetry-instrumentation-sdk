// SpanType is a non official open-telemetry key, only supported by Datadog, to help categorize traces.
// Documentation: https://github.com/open-telemetry/opentelemetry-rust/blob/ccb510fbd6fdef9694e3b751fd01dbe33c7345c0/opentelemetry-datadog/src/lib.rs#L29-L30
// Usage: It should be informed as span.type span key
// Reference: https://github.com/DataDog/dd-trace-go/blob/352b090d4f90527d35a8ad535b97689e346589c8/ddtrace/ext/app_types.go#L31-L81
#[allow(dead_code)]
pub enum SpanType {
    Web,
    Http,
    Sql,
    Cassandra,
    Redis,
    Memcached,
    Mongodb,
    Elasticsearch,
    Leveldb,
    Dns,
    Queue,
    Consul,
    Graphql,
}

impl ToString for SpanType {
    fn to_string(&self) -> String {
        match self {
            SpanType::Web => "web",
            SpanType::Http => "http",
            SpanType::Sql => "sql",
            SpanType::Cassandra => "cassandra",
            SpanType::Redis => "redis",
            SpanType::Memcached => "memcached",
            SpanType::Mongodb => "mongodb",
            SpanType::Elasticsearch => "elasticsearch",
            SpanType::Leveldb => "leveldb",
            SpanType::Dns => "dns",
            SpanType::Queue => "queue",
            SpanType::Consul => "consul",
            SpanType::Graphql => "graphql",
        }
        .to_owned()
    }
}
