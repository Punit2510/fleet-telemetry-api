use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_json::json;
use uuid::Uuid;

fn telemetry_payload_benchmark(c: &mut Criterion) {
    let device = Uuid::new_v4();
    c.bench_function("serialize_100_telemetry_events", |b| b.iter(|| {
        let events: Vec<_> = (0..100).map(|i| json!({
            "device_id": device,
            "observed_at": "2026-11-25T10:00:00Z",
            "metric": "engine_temperature",
            "value": 70.0 + (i as f64 / 10.0),
            "unit": "celsius"
        })).collect();
        black_box(serde_json::to_vec(&events).unwrap())
    }));
}
criterion_group!(benches, telemetry_payload_benchmark);
criterion_main!(benches);
