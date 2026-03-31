use criterion::{black_box, criterion_group, criterion_main, Criterion};
use oxide_seo_lib::crawler::frontier::{hash_url, normalize_url, FrontierEntry, UrlFrontier};

fn bench_normalize_url(c: &mut Criterion) {
    c.bench_function("normalize_url", |b| {
        b.iter(|| {
            normalize_url(
                black_box("https://Example.COM:443/path/to/page?z=1&a=2&m=3#fragment"),
                true,
            )
        });
    });
}

fn bench_hash_url(c: &mut Criterion) {
    let url = "https://example.com/some/page/with/a/long/path";
    c.bench_function("hash_url_blake3", |b| {
        b.iter(|| hash_url(black_box(url)));
    });
}

fn bench_frontier_push_pop(c: &mut Criterion) {
    c.bench_function("frontier_push_pop_1000", |b| {
        b.iter(|| {
            let mut frontier = UrlFrontier::new(10_000);
            for i in 0..1000 {
                let url = format!("https://example.com/page/{}", i);
                let hash = hash_url(&url);
                frontier.push(FrontierEntry {
                    url,
                    url_hash: hash,
                    depth: (i % 5) as u32,
                    priority: 100 - (i % 5) as i32,
                    source_page_id: None,
                });
            }
            for _ in 0..1000 {
                black_box(frontier.pop());
            }
        });
    });
}

criterion_group!(benches, bench_normalize_url, bench_hash_url, bench_frontier_push_pop);
criterion_main!(benches);
