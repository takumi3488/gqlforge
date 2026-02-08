use std::path::Path;

use anyhow::Result;
use criterion::Criterion;
use gqlforge::core::blueprint::GrpcMethod;
use gqlforge::core::grpc::protobuf::ProtobufSet;
use rand::Rng;
use serde_json::{json, Value};

const PROTO_DIR: &str = "benches/grpc";
const PROTO_FILE: &str = "dummy.proto";
const SERVICE_NAME: &str = "dummy.DummyService.GetDummy";
const N: usize = 1000;
const M: usize = 100;

fn create_dummy_value(n: usize, m: usize) -> Result<Value> {
    let mut rng = rand::rng();
    let mut ints = vec![0i32; n];
    let mut floats = vec![0f32; n];
    let mut flags = vec![false; n];
    let names: Vec<String> = (0..n)
        .map(|_| {
            let mut chars = vec![' '; m];

            rng.fill(chars.as_mut_slice());

            Ok(chars.into_iter().collect::<String>())
        })
        .collect::<Result<_>>()?;

    rng.fill(ints.as_mut_slice());
    rng.fill(floats.as_mut_slice());
    rng.fill(flags.as_mut_slice());

    let value = json!({
        "ints": ints,
        "floats": floats,
        "flags": flags,
        "names": names,
    });

    Ok(value)
}

pub fn benchmark_convert_output(c: &mut Criterion) {
    let proto_file_path = Path::new(PROTO_DIR).join(PROTO_FILE);
    let file_descriptor_set = protox::compile([proto_file_path], ["."]).unwrap();
    let protobuf_set = ProtobufSet::from_proto_file(file_descriptor_set).unwrap();
    let method = GrpcMethod::try_from(SERVICE_NAME).unwrap();
    let service = protobuf_set.find_service(&method).unwrap();
    let protobuf_operation = service.find_operation(&method).unwrap();

    let dummy_value = create_dummy_value(N, M).unwrap();
    let msg = protobuf_operation
        .convert_input(&dummy_value.to_string())
        .unwrap();

    c.bench_function("test_batched_body", |b| {
        b.iter(|| {
            std::hint::black_box(
                protobuf_operation
                    .convert_output::<serde_json::Value>(&msg)
                    .unwrap(),
            );
        })
    });
}
