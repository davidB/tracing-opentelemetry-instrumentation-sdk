use std::collections::BTreeMap;

pub(crate) fn cnv_attributes(
    attributes: &[opentelemetry_proto::tonic::common::v1::KeyValue],
) -> BTreeMap<String, String> {
    attributes
        .iter()
        .map(|kv| (kv.key.to_string(), format!("{:?}", kv.value)))
        .collect::<BTreeMap<String, String>>()
}
