use std::cmp::Ordering;
use std::f64::consts::PI;

use ndarray::prelude::*;
use ndarray_linalg::Norm;
use rand::distributions::{Standard, Uniform, WeightedIndex};
use rand::prelude::*;
use rayon::prelude::*;

#[derive(Debug)]
enum Error {
    NanInTargetFunction,
}

fn evolution(
    f: impl Fn(&ArrayView1<f64>) -> f64 + Sync + Send,
    bounds: impl AsRef<[[f64; 2]]>,
    population_size: usize,
    max_iter: usize,
    f_eps_min: f64,
    learn_rate: f64,
    mutation_probability: f64,
    mutation_scale: f64,
) -> Result<EvoResult, Error> {
    let mut rng = thread_rng();
    let sampling_bounds = bounds.as_ref().iter().map(|[l, u]| Uniform::new(l, u));
    let mut population: Vec<Agent> = (0..population_size * 2)
        .map(|_| {
            let location: Array1<f64> = sampling_bounds
                .clone()
                .map(|s| s.sample(&mut rng))
                .collect();
            let fitness = f(&location.view());
            Agent { fitness, location }
        })
        .collect();
    population
        .sort_unstable_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(Ordering::Equal));

    let mut children = Vec::new();

    let mut iteration = 0;
    let mut f_eps = population[0].fitness - population[population_size].fitness;
    while iteration < max_iter && f_eps > f_eps_min {
        if population.iter().any(|a| a.fitness.is_nan()) {
            return Err(Error::NanInTargetFunction);
        }
        let distances: Vec<f64> = population
            .par_iter()
            .take(population_size)
            .map(|agent| agent.location.norm_l2())
            .collect();
        let distance_matrix =
            Array2::from_shape_fn((population_size, population_size), |(i, j)| {
                distance_func(&distances, i, j)
            });
        let partner_choice: Vec<&Agent> = distance_matrix
            .axis_iter(Axis(0))
            .map(|ax| &population[WeightedIndex::new(&ax).unwrap().sample(&mut rng)])
            .collect();

        let recombination_weights: Vec<f64> = (&mut rng)
            .sample_iter(Standard)
            .take(population_size)
            .collect();
        let mutations: Vec<Option<Array1<f64>>> = (0..population_size)
            .map(|_| {
                sampling_bounds
                    .clone()
                    .map(|s| {
                        if rng.gen_bool(mutation_probability) {
                            Some(s.sample(&mut rng))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .collect();

        population
            .par_iter()
            .take(population_size)
            .zip(partner_choice)
            .zip(recombination_weights)
            .zip(mutations)
            .map(|(((p1, p2), w1), m)| {
                make_child(
                    &population[0],
                    learn_rate,
                    p1,
                    p2,
                    w1,
                    m,
                    mutation_scale,
                    &f,
                )
            })
            .collect_into_vec(&mut children);
        population[population_size..].swap_with_slice(&mut children);

        population
            .sort_unstable_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(Ordering::Equal));

        f_eps = population[0].fitness - population[population_size - 1].fitness;
        iteration += 1;
    }

    Ok(EvoResult {
        f_eps,
        agent: population.swap_remove(0),
        n_iter: iteration,
    })
}

#[inline(always)]
fn make_child(
    p0: &Agent,
    learn_rate: f64,
    p1: &Agent,
    p2: &Agent,
    w1: f64,
    m: Option<Array1<f64>>,
    mutation_scale: f64,
    f: impl Fn(&ArrayView1<f64>) -> f64,
) -> Agent {
    let location_unmutated = &(&p1.location * w1) + &(&p2.location * (1. - w1));

    let location_unlearned = if let Some(mutation) = m {
        &location_unmutated + &(&mutation - &location_unmutated) * mutation_scale
    } else {
        location_unmutated
    };
    let location = &location_unlearned + (&p0.location - &location_unlearned) * learn_rate;
    let fitness = f(&location.view());
    Agent { location, fitness }
}

#[inline(always)]
fn distance_func(distances: &Vec<f64>, i: usize, j: usize) -> f64 {
    if i == j {
        0.
    } else {
        (-(distances[i].powi(2) + distances[j].powi(2) - 2f64 * distances[i] * distances[j])).exp()
    }
}

#[derive(Debug)]
struct EvoResult {
    agent: Agent,
    n_iter: usize,
    f_eps: f64,
}

#[derive(Debug)]
struct Agent {
    location: Array1<f64>,
    fitness: f64,
}

fn main() {
    println!("Hello, world!");
    // let v = Array1::from_vec(vec![])
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
    match evolution(
        f,
        bounds,
        population_size,
        max_iter,
        f_eps,
        learn_rate,
        mutation_probability,
        mutation_scale,
    ) {
        Ok(res) => println!("success! found {:?}", res),
        Err(e) => println!("errored: {:?}", e),
    }
}
