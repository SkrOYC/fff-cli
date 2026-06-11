use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use hashlink::LruCache;
use std::path::PathBuf;

const CACHE_SIZE_ENTRIES: usize = 10_000;
const CACHE_SIZE_BYTES: usize = 256 * 1024 * 1024;
const AVG_FILE_SIZE: usize = 4096;

struct SimpleLruCache {
    cache: LruCache<PathBuf, Vec<u8>>,
}

impl SimpleLruCache {
    fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(capacity),
        }
    }

    fn insert(&mut self, path: PathBuf, content: Vec<u8>) {
        self.cache.insert(path, content);
    }

    fn get(&mut self, path: &PathBuf) -> Option<&Vec<u8>> {
        self.cache.get(path)
    }
}

struct BatchEvictionCache {
    cache: LruCache<PathBuf, Vec<u8>>,
    budget_bytes: usize,
    current_bytes: usize,
}

impl BatchEvictionCache {
    fn new(budget_bytes: usize) -> Self {
        Self {
            cache: LruCache::new_unbounded(),
            budget_bytes,
            current_bytes: 0,
        }
    }

    fn insert(&mut self, path: PathBuf, content: Vec<u8>) {
        let content_size = content.len();

        if let Some(old) = self.cache.insert(path, content) {
            self.current_bytes = self.current_bytes.saturating_sub(old.len());
            self.current_bytes += content_size;
        } else {
            self.current_bytes += content_size;
        }

        if self.current_bytes > self.budget_bytes {
            let target = self.budget_bytes * 80 / 100;
            while self.current_bytes > target {
                if let Some((_, evicted)) = self.cache.remove_lru() {
                    self.current_bytes = self.current_bytes.saturating_sub(evicted.len());
                } else {
                    break;
                }
            }
        }
    }

    fn get(&mut self, path: &PathBuf) -> Option<&Vec<u8>> {
        self.cache.get(path)
    }
}

fn generate_test_data(count: usize, avg_size: usize) -> Vec<(PathBuf, Vec<u8>)> {
    (0..count)
        .map(|i| {
            let path = PathBuf::from(format!("/test/file_{i}.txt"));
            let size = avg_size + (i % 100) * 10;
            let content = vec![b'x'; size];
            (path, content)
        })
        .collect()
}

fn bench_insert_simple(c: &mut Criterion) {
    let data = generate_test_data(CACHE_SIZE_ENTRIES, AVG_FILE_SIZE);

    let mut group = c.benchmark_group("insert_simple");
    group.throughput(Throughput::Elements(CACHE_SIZE_ENTRIES as u64));
    group.bench_function("simple_lru", |b| {
        b.iter(|| {
            let mut cache = SimpleLruCache::new(CACHE_SIZE_ENTRIES);
            for (path, content) in &data {
                cache.insert(path.clone(), content.clone());
            }
            cache
        });
    });
    group.finish();
}

fn bench_insert_byte_tracked(c: &mut Criterion) {
    let data = generate_test_data(CACHE_SIZE_ENTRIES, AVG_FILE_SIZE);

    let mut group = c.benchmark_group("insert_byte_tracked");
    group.throughput(Throughput::Elements(CACHE_SIZE_ENTRIES as u64));
    group.bench_function("batch_eviction", |b| {
        b.iter(|| {
            let mut cache = BatchEvictionCache::new(CACHE_SIZE_BYTES);
            for (path, content) in &data {
                cache.insert(path.clone(), content.clone());
            }
            cache
        });
    });
    group.finish();
}

fn bench_get_simple(c: &mut Criterion) {
    let data = generate_test_data(CACHE_SIZE_ENTRIES, AVG_FILE_SIZE);
    let mut cache = SimpleLruCache::new(CACHE_SIZE_ENTRIES);
    for (path, content) in &data {
        cache.insert(path.clone(), content.clone());
    }
    let keys: Vec<PathBuf> = data.iter().map(|(p, _)| p.clone()).collect();

    let mut group = c.benchmark_group("get_simple");
    group.throughput(Throughput::Elements(CACHE_SIZE_ENTRIES as u64));
    group.bench_function("simple_lru", |b| {
        b.iter(|| {
            for key in &keys {
                cache.get(key);
            }
        });
    });
    group.finish();
}

fn bench_get_byte_tracked(c: &mut Criterion) {
    let data = generate_test_data(CACHE_SIZE_ENTRIES, AVG_FILE_SIZE);
    let mut cache = BatchEvictionCache::new(CACHE_SIZE_BYTES);
    for (path, content) in &data {
        cache.insert(path.clone(), content.clone());
    }
    let keys: Vec<PathBuf> = data.iter().map(|(p, _)| p.clone()).collect();

    let mut group = c.benchmark_group("get_byte_tracked");
    group.throughput(Throughput::Elements(CACHE_SIZE_ENTRIES as u64));
    group.bench_function("batch_eviction", |b| {
        b.iter(|| {
            for key in &keys {
                cache.get(key);
            }
        });
    });
    group.finish();
}

fn bench_eviction_under_pressure(c: &mut Criterion) {
    let budget = 10 * 1024 * 1024;
    let data = generate_test_data(10_000, AVG_FILE_SIZE);

    let mut group = c.benchmark_group("eviction_under_pressure");
    group.bench_function("evict_to_budget", |b| {
        b.iter(|| {
            let mut cache = BatchEvictionCache::new(budget);
            for (path, content) in &data {
                cache.insert(path.clone(), content.clone());
            }
            assert!(cache.current_bytes <= budget);
            cache
        });
    });
    group.finish();
}

fn bench_memory_pressure_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_pressure");

    group.bench_function("parse_proc_meminfo", |b| {
        b.iter(|| {
            if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
                for line in content.lines() {
                    if line.starts_with("MemAvailable:") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let _kb: u64 = parts[1].parse().unwrap_or(0);
                        }
                    }
                }
            }
        });
    });

    group.finish();
}

fn bench_overhead_comparison(c: &mut Criterion) {
    let data = generate_test_data(5_000, AVG_FILE_SIZE);

    let mut group = c.benchmark_group("overhead_comparison");
    group.throughput(Throughput::Bytes((5_000 * AVG_FILE_SIZE) as u64));

    group.bench_function("simple_lru_5k", |b| {
        b.iter(|| {
            let mut cache = SimpleLruCache::new(5_000);
            for (path, content) in &data {
                cache.insert(path.clone(), content.clone());
            }
            cache
        });
    });

    group.bench_function("batch_eviction_5k", |b| {
        b.iter(|| {
            let mut cache = BatchEvictionCache::new(CACHE_SIZE_BYTES);
            for (path, content) in &data {
                cache.insert(path.clone(), content.clone());
            }
            cache
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_insert_simple,
    bench_insert_byte_tracked,
    bench_get_simple,
    bench_get_byte_tracked,
    bench_eviction_under_pressure,
    bench_memory_pressure_detection,
    bench_overhead_comparison,
);
criterion_main!(benches);
