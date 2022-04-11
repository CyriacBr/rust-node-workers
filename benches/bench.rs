use std::process::Command;

use criterion::{black_box, criterion_group, criterion_main, Criterion, SamplingMode, Throughput};
use node_workers::WorkerPool;

fn bench_fast_binary(c: &mut Criterion) {
  let mut group = c.benchmark_group("fast node binary");
  group.sample_size(30);
  group.throughput(Throughput::Elements(1 as u64));
  group.sampling_mode(SamplingMode::Flat);

  group.bench_function("standard commands", |b| {
    b.iter(|| {
      Command::new("node")
        .arg("benches/workers/fast-inner")
        .arg("30")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    })
  });
  let mut pool = WorkerPool::setup(1);
  group.bench_function("worker pool", |b| {
    b.iter(|| {
      pool
        .run_task::<(), _>("benches/workers/fast", "fib", black_box(vec![30]))
        .unwrap();
    })
  });

  group.finish();
}

fn bench_slow_binary(c: &mut Criterion) {
  let mut group = c.benchmark_group("slow node binary");
  group.sample_size(10);
  group.throughput(Throughput::Elements(1 as u64));
  group.sampling_mode(SamplingMode::Flat);

  group.bench_function("standard commands", |b| {
    b.iter(|| {
      Command::new("node")
        .arg("benches/workers/slow-inner")
        .arg("30")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    })
  });
  let mut pool = WorkerPool::setup(1);
  group.bench_function("worker pool", |b| {
    b.iter(|| {
      pool
        .run_task::<(), _>("benches/workers/slow", "fib", black_box(vec![30]))
        .unwrap();
    })
  });

  group.finish();
}

criterion_group!(benches, bench_fast_binary, bench_slow_binary);
criterion_main!(benches);
