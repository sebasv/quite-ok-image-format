use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ndarray::prelude::*;
use rustevo::evolution;
use std::f64::consts::PI;

fn criterion_benchmark(c: &mut Criterion) {
    let f = |x: &ArrayView1<f64>| {
        let levi = f64::sin(x[0] * 3. * PI).powi(2)
            + (x[0] - 1.).powi(2) * (1. + f64::sin(x[1] * 3. * PI).powi(2))
            + (x[1] - 1.).powi(2) * (1. + f64::sin(x[1] * 2. * PI).powi(2));
        -levi
    };
    let bounds = [[-10., 10.], [-10., 10.]];
    let population_size = 100;
    let max_iter = 100;
    let learn_rate = 0.01;
    let f_eps = 1e-12;
    let mutation_probability = 0.1;
    let mutation_scale = 0.1;
    let seed = 42;
    c.bench_function("fib 20", |b| {
        b.iter(|| {
            evolution(
                f,
                bounds,
                population_size,
                max_iter,
                f_eps,
                learn_rate,
                mutation_probability,
                mutation_scale,
                seed,
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
