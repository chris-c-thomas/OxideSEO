use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_html_parse(c: &mut Criterion) {
    let html = include_bytes!("../../tests/fixtures/index.html");

    c.bench_function("parse_html_lol_html", |b| {
        b.iter(|| {
            oxide_seo_lib::crawler::parser::parse_html(
                black_box(html),
                "https://test.local/",
                "test.local",
            )
        });
    });
}

criterion_group!(benches, bench_html_parse);
criterion_main!(benches);
