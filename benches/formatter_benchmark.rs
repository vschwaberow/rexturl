use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rexturl::formatter::{UrlRecord, to_record};

fn create_test_records(count: usize) -> Vec<UrlRecord> {
    let test_urls = [
        "https://www.example.com/path?query=value#fragment",
        "https://api.github.com/repos/user/repo/releases/latest",
        "https://cdn.jsdelivr.net/npm/package@1.0.0/dist/bundle.js",
        "mongodb://user:password@cluster.mongodb.net:27017/database?ssl=true",
        "redis://localhost:6379/0",
    ];
    
    (0..count)
        .map(|i| {
            let url = test_urls[i % test_urls.len()];
            to_record(url).expect("Valid URL")
        })
        .collect()
}

fn bench_field_lookup(c: &mut Criterion) {
    let records = create_test_records(1000);
    let fields = vec!["scheme", "domain", "path", "query"];
    
    c.bench_function("field_lookup", |b| {
        b.iter(|| {
            for record in &records {
                for field in &fields {
                    let _ = black_box(record.get_field(field));
                }
            }
        })
    });
}

fn bench_url_record_creation(c: &mut Criterion) {
    let test_urls = [
        "https://www.example.com/path?query=value#fragment",
        "https://api.github.com/repos/user/repo/releases/latest",
        "https://cdn.jsdelivr.net/npm/package@1.0.0/dist/bundle.js",
    ];
    
    c.bench_function("url_record_creation", |b| {
        b.iter(|| {
            for url in test_urls.iter() {
                let record = black_box(to_record(url));
                black_box(record);
            }
        })
    });
}

fn bench_large_dataset_creation(c: &mut Criterion) {
    c.bench_function("create_1k_records", |b| {
        b.iter(|| {
            let records = black_box(create_test_records(1000));
            black_box(records);
        })
    });
}

criterion_group!(
    benches,
    bench_field_lookup,
    bench_url_record_creation,
    bench_large_dataset_creation
);
criterion_main!(benches);