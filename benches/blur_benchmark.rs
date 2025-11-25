use criterion::{Criterion, criterion_group, criterion_main};
use image_blur::{ImageSoA, Pixel, blur_cache_optimized, blur_naive, blur_separable};
use std::hint::black_box;

fn create_test_image_aos() -> Vec<Pixel> {
    vec![
        Pixel { 
            r: 100, 
            g: 150, 
            b: 200,
        }; 
        512 * 512
    ]
}

fn create_test_image_soa() -> ImageSoA {
    ImageSoA {
        width: 512,
        height: 512,
        r: vec![100; 512 * 512],
        g: vec![150; 512 * 512],
        b: vec![200; 512 * 512],
    }
}

fn benchmark_blur_naive(c: &mut Criterion) {
    let image = create_test_image_aos();
    c.bench_function("blur_naive", |b| {
        b.iter(|| blur_naive(black_box(&image), 512, 512))
    });
}

fn benchmark_blur_cache_optimized(c: &mut Criterion) {
    let image = create_test_image_soa();
    c.bench_function("blur_cache_optimized", |b| {
        b.iter(|| blur_cache_optimized(black_box(&image)))
    });
}

fn benchmark_blur_separable(c: &mut Criterion) {
    let image = create_test_image_soa();
    c.bench_function("blur_separable", |b| {
        b.iter(|| blur_separable(black_box(&image)))
    });
}

criterion_group!(
    benches, 
    benchmark_blur_naive, 
    benchmark_blur_cache_optimized,
    benchmark_blur_separable,
);
criterion_main!(benches);
