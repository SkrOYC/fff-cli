use bytes::{Bytes, BytesMut};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use serde::{Deserialize, Serialize};
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MatchItem {
    path: String,
    line_number: u32,
    column: u32,
    line_content: String,
    match_ranges: Vec<(u32, u32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Notification {
    jsonrpc: String,
    method: String,
    params: NotificationParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NotificationParams {
    items: Vec<MatchItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FinalResponse {
    jsonrpc: String,
    id: u64,
    result: ResponseResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResponseResult {
    total_matched: u64,
    elapsed_ms: u64,
}

fn generate_match_items(count: usize) -> Vec<MatchItem> {
    (0..count)
        .map(|i| MatchItem {
            path: format!("src/module_{i}/file_{i}.rs"),
            line_number: (i * 7) as u32,
            column: 8,
            line_content: format!("    let item_{i} = process(data_{i}); // TODO: optimize"),
            match_ranges: vec![(40, 44)],
        })
        .collect()
}

fn generate_notification(items: Vec<MatchItem>) -> Notification {
    Notification {
        jsonrpc: "2.0".to_string(),
        method: "result".to_string(),
        params: NotificationParams { items },
    }
}

fn generate_final_response(id: u64, total: u64) -> FinalResponse {
    FinalResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: ResponseResult {
            total_matched: total,
            elapsed_ms: 52,
        },
    }
}

fn bench_raw_json_serialize(c: &mut Criterion) {
    let items = generate_match_items(1000);
    let notification = generate_notification(items);

    let mut group = c.benchmark_group("raw_json");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("serialize_1000_items", |b| {
        b.iter(|| serde_json::to_string(&notification).unwrap());
    });

    group.finish();
}

fn bench_raw_json_roundtrip(c: &mut Criterion) {
    let items = generate_match_items(1000);
    let notification = generate_notification(items);
    let json = serde_json::to_string(&notification).unwrap();

    let mut group = c.benchmark_group("raw_json");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("roundtrip_1000_items", |b| {
        b.iter(|| {
            let _: Notification = serde_json::from_str(&json).unwrap();
        });
    });

    group.finish();
}

fn bench_length_prefixed_encode(c: &mut Criterion) {
    let items = generate_match_items(1000);
    let notification = generate_notification(items);
    let json = serde_json::to_vec(&notification).unwrap();

    let mut codec = LengthDelimitedCodec::builder()
        .max_frame_length(100 * 1024 * 1024)
        .length_field_length(4)
        .big_endian()
        .new_codec();

    let mut group = c.benchmark_group("length_prefixed");
    group.throughput(Throughput::Bytes(json.len() as u64));

    group.bench_function("encode_1000_items", |b| {
        b.iter(|| {
            let mut dst = BytesMut::with_capacity(json.len() + 4);
            codec.encode(Bytes::from(json.clone()), &mut dst).unwrap();
            dst
        });
    });

    group.finish();
}

fn bench_length_prefixed_decode(c: &mut Criterion) {
    let items = generate_match_items(1000);
    let notification = generate_notification(items);
    let json = serde_json::to_vec(&notification).unwrap();

    let mut codec = LengthDelimitedCodec::builder()
        .max_frame_length(100 * 1024 * 1024)
        .length_field_length(4)
        .big_endian()
        .new_codec();

    let mut encoded = BytesMut::with_capacity(json.len() + 4);
    codec.encode(Bytes::from(json), &mut encoded).unwrap();

    let mut group = c.benchmark_group("length_prefixed");
    group.throughput(Throughput::Bytes(encoded.len() as u64));

    group.bench_function("decode_1000_items", |b| {
        b.iter(|| {
            let mut src = encoded.clone();
            let frame = codec.decode(&mut src).unwrap().unwrap();
            let _: Notification = serde_json::from_slice(&frame).unwrap();
        });
    });

    group.finish();
}

fn bench_length_prefixed_roundtrip(c: &mut Criterion) {
    let items = generate_match_items(1000);
    let notification = generate_notification(items);
    let json = serde_json::to_vec(&notification).unwrap();

    let mut codec = LengthDelimitedCodec::builder()
        .max_frame_length(100 * 1024 * 1024)
        .length_field_length(4)
        .big_endian()
        .new_codec();

    let mut group = c.benchmark_group("length_prefixed");
    group.throughput(Throughput::Bytes(json.len() as u64));

    group.bench_function("roundtrip_1000_items", |b| {
        b.iter(|| {
            let mut dst = BytesMut::with_capacity(json.len() + 4);
            codec.encode(Bytes::from(json.clone()), &mut dst).unwrap();
            let frame = codec.decode(&mut dst).unwrap().unwrap();
            let _: Notification = serde_json::from_slice(&frame).unwrap();
        });
    });

    group.finish();
}

fn bench_batch_sizes(c: &mut Criterion) {
    let all_items = generate_match_items(10_000);

    let mut codec = LengthDelimitedCodec::builder()
        .max_frame_length(100 * 1024 * 1024)
        .length_field_length(4)
        .big_endian()
        .new_codec();

    let mut group = c.benchmark_group("batch_size_comparison");
    group.throughput(Throughput::Elements(10_000));

    for batch_size in [10, 100, 1000] {
        let batch_count = 10_000 / batch_size;
        group.bench_function(format!("batch_{batch_size}_notifications"), |b| {
            b.iter(|| {
                let mut total_bytes = 0;
                for batch_idx in 0..batch_count {
                    let start = batch_idx * batch_size;
                    let end = start + batch_size;
                    let batch_items = all_items[start..end].to_vec();
                    let notification = generate_notification(batch_items);
                    let json = serde_json::to_vec(&notification).unwrap();
                    let mut dst = BytesMut::with_capacity(json.len() + 4);
                    codec.encode(Bytes::from(json), &mut dst).unwrap();
                    total_bytes += dst.len();
                }
                total_bytes
            });
        });
    }

    group.bench_function("single_notification_10000", |b| {
        b.iter(|| {
            let notification = generate_notification(all_items.clone());
            let json = serde_json::to_vec(&notification).unwrap();
            let mut dst = BytesMut::with_capacity(json.len() + 4);
            codec.encode(Bytes::from(json), &mut dst).unwrap();
            dst.len()
        });
    });

    group.finish();
}

fn bench_streaming_throughput(c: &mut Criterion) {
    let all_items = generate_match_items(100_000);
    let batch_size = 100;

    let mut codec = LengthDelimitedCodec::builder()
        .max_frame_length(100 * 1024 * 1024)
        .length_field_length(4)
        .big_endian()
        .new_codec();

    let mut group = c.benchmark_group("streaming_throughput");
    group.throughput(Throughput::Elements(100_000));
    group.sample_size(10);

    group.bench_function("100k_items_batch_100", |b| {
        b.iter(|| {
            let mut total_bytes = 0;
            let batch_count = 100_000 / batch_size;
            for batch_idx in 0..batch_count {
                let start = batch_idx * batch_size;
                let end = start + batch_size;
                let batch_items = all_items[start..end].to_vec();
                let notification = generate_notification(batch_items);
                let json = serde_json::to_vec(&notification).unwrap();
                let mut dst = BytesMut::with_capacity(json.len() + 4);
                codec.encode(Bytes::from(json), &mut dst).unwrap();
                total_bytes += dst.len();
            }
            let final_resp = generate_final_response(1, 100_000);
            let json = serde_json::to_vec(&final_resp).unwrap();
            let mut dst = BytesMut::with_capacity(json.len() + 4);
            codec.encode(Bytes::from(json), &mut dst).unwrap();
            total_bytes += dst.len();
            total_bytes
        });
    });

    group.finish();
}

fn bench_overhead_comparison(c: &mut Criterion) {
    let items = generate_match_items(1000);
    let notification = generate_notification(items);
    let json_bytes = serde_json::to_vec(&notification).unwrap();

    let mut codec = LengthDelimitedCodec::builder()
        .max_frame_length(100 * 1024 * 1024)
        .length_field_length(4)
        .big_endian()
        .new_codec();

    let mut group = c.benchmark_group("overhead_comparison");
    group.throughput(Throughput::Bytes(json_bytes.len() as u64));

    group.bench_function("raw_json_only", |b| {
        b.iter(|| {
            let _ = serde_json::to_vec(&notification).unwrap();
        });
    });

    group.bench_function("length_prefixed_framing", |b| {
        b.iter(|| {
            let json = serde_json::to_vec(&notification).unwrap();
            let mut dst = BytesMut::with_capacity(json.len() + 4);
            codec.encode(Bytes::from(json), &mut dst).unwrap();
            dst
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_raw_json_serialize,
    bench_raw_json_roundtrip,
    bench_length_prefixed_encode,
    bench_length_prefixed_decode,
    bench_length_prefixed_roundtrip,
    bench_batch_sizes,
    bench_streaming_throughput,
    bench_overhead_comparison,
);
criterion_main!(benches);
