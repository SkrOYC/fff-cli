use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer as new_debouncer_mini;
use std::fs;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tempfile::TempDir;

fn bench_burst_event_count(c: &mut Criterion) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().to_path_buf();

    let mut group = c.benchmark_group("burst_events");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("create_1000_files", |b| {
        b.iter(|| {
            let sub_dir = path.join(format!("burst_{}", Instant::now().elapsed().as_nanos()));
            fs::create_dir_all(&sub_dir).unwrap();
            for i in 0..1000 {
                fs::write(sub_dir.join(format!("file_{i}.txt")), "x").unwrap();
            }
            fs::remove_dir_all(&sub_dir).ok();
        });
    });

    group.finish();
}

fn bench_debouncer_mini_latency(c: &mut Criterion) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().to_path_buf();

    let mut group = c.benchmark_group("debouncer_mini");

    group.bench_function("single_event_latency", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::ZERO;
            for _ in 0..iters {
                let (tx, rx) = mpsc::channel();
                let mut debouncer = new_debouncer_mini(Duration::from_millis(100), tx).unwrap();
                debouncer
                    .watcher()
                    .watch(&path, RecursiveMode::Recursive)
                    .unwrap();

                let start = Instant::now();
                fs::write(path.join("test.txt"), "hello").unwrap();

                let _ = rx
                    .recv_timeout(Duration::from_secs(2))
                    .expect("debouncer should emit event within timeout");
                total += start.elapsed();

                fs::remove_file(path.join("test.txt")).ok();
            }
            total
        });
    });

    group.finish();
}

fn bench_burst_coalescing_mini(c: &mut Criterion) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().to_path_buf();

    let mut group = c.benchmark_group("burst_coalescing_mini");

    group.bench_function("1000_files_debounced", |b| {
        b.iter(|| {
            let (tx, rx) = mpsc::channel();
            let mut debouncer = new_debouncer_mini(Duration::from_millis(500), tx).unwrap();
            debouncer
                .watcher()
                .watch(&path, RecursiveMode::Recursive)
                .unwrap();

            let start = Instant::now();

            for i in 0..1000 {
                fs::write(path.join(format!("file_{i}.txt")), "x").unwrap();
            }

            let mut event_count = 0;
            while rx.recv_timeout(Duration::from_secs(3)).is_ok() {
                event_count += 1;
            }

            let elapsed = start.elapsed();

            for i in 0..1000 {
                fs::remove_file(path.join(format!("file_{i}.txt"))).ok();
            }

            (event_count, elapsed)
        });
    });

    group.finish();
}

fn bench_inotify_watch_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("inotify_watches");

    group.bench_function("watch_nix_store", |b| {
        b.iter_custom(|iters| {
            let nix_store = std::path::Path::new("/nix/store");
            if !nix_store.exists() {
                return Duration::ZERO;
            }

            let mut total = Duration::ZERO;
            for _ in 0..iters {
                let (tx, _rx) = mpsc::channel();
                let mut debouncer = new_debouncer_mini(Duration::from_millis(500), tx).unwrap();

                let start = Instant::now();
                let _ = debouncer
                    .watcher()
                    .watch(nix_store, RecursiveMode::Recursive);
                total += start.elapsed();
            }
            total
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_burst_event_count,
    bench_debouncer_mini_latency,
    bench_burst_coalescing_mini,
    bench_inotify_watch_count,
);
criterion_main!(benches);
