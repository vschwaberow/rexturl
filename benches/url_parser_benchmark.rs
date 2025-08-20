use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rexturl::url::Url;

const TEST_URLS: &[&str] = &[
    "https://www.example.com",
    "https://user:pass@blog.example.com:8080/path/to/page?param=value&other=test#section",
    "http://[::1]:8080/api/v1/users",
    "ftp://files.example.org/downloads/file.zip",
    "https://api.github.com/repos/user/repo/releases/latest",
    "https://cdn.jsdelivr.net/npm/package@1.0.0/dist/bundle.js",
    "mongodb://user:password@cluster.mongodb.net:27017/database?ssl=true",
    "redis://localhost:6379/0",
    "postgresql://user:password@localhost:5432/database",
    "mysql://user:password@localhost:3306/database",
];

fn bench_fast_url_parsing(c: &mut Criterion) {
    c.bench_function("fast_url_parsing", |b| {
        b.iter(|| {
            for url_str in TEST_URLS {
                let _ = black_box(Url::parse(url_str));
            }
        })
    });
}

fn bench_fast_url_component_access(c: &mut Criterion) {
    let parsed_urls: Vec<Url> = TEST_URLS
        .iter()
        .filter_map(|url_str| Url::parse(url_str).ok())
        .collect();

    c.bench_function("fast_url_component_access", |b| {
        b.iter(|| {
            for url in &parsed_urls {
                let _ = black_box(url.scheme());
                let _ = black_box(url.host_str());
                let _ = black_box(url.port());
                let _ = black_box(url.path());
                let _ = black_box(url.query());
                let _ = black_box(url.fragment());
            }
        })
    });
}

fn bench_fast_url_full_pipeline(c: &mut Criterion) {
    c.bench_function("fast_url_full_pipeline", |b| {
        b.iter(|| {
            for url_str in TEST_URLS {
                if let Ok(url) = Url::parse(url_str) {
                    let _ = black_box(url.scheme());
                    let _ = black_box(url.host_str());
                    let _ = black_box(url.port());
                    let _ = black_box(url.path());
                    let _ = black_box(url.query());
                    let _ = black_box(url.fragment());
                }
            }
        })
    });
}

fn bench_fast_url_with_scheme_prefix(c: &mut Criterion) {
    c.bench_function("fast_url_with_scheme_prefix", |b| {
        b.iter(|| {
            for url_str in TEST_URLS {
                let url_with_scheme = if !url_str.contains("://") {
                    format!("https://{url_str}")
                } else {
                    url_str.to_string()
                };
                let _ = black_box(Url::parse(&url_with_scheme));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_fast_url_parsing,
    bench_fast_url_component_access,
    bench_fast_url_full_pipeline,
    bench_fast_url_with_scheme_prefix
);
criterion_main!(benches);
