use std::cmp::Ordering;

use ndarray::prelude::*;
use ndarray_linalg::Norm;
use rand::distributions::{Uniform, WeightedIndex};
use rand::prelude::*;
use rayon::prelude::*;

#[derive(Debug)]
pub enum Error {
    NanInTargetFunction,
    WeightsAreZero,
    RngFailure,
}

pub fn evolution(
    f: impl Fn(&ArrayView1<f64>) -> f64 + Sync + Send,
    bounds: impl AsRef<[[f64; 2]]>,
    population_size: usize,
    max_iter: usize,
    f_eps_min: f64,
    learn_rate: f64,
    mutation_probability: f64,
    mutation_scale: f64,
    seed: u64,
) -> Result<EvoResult, Error> {
    let mut source_rng = StdRng::seed_from_u64(seed);
    let mut rngs = (0..population_size)
        .map(|_| StdRng::from_rng(&mut source_rng))
        .collect::<Result<Vec<StdRng>, _>>()
        .or(Err(Error::RngFailure))?;

    let sampling_bounds = bounds.as_ref().iter().map(|[l, u]| Uniform::new(l, u));
    let mut population: Vec<Agent> = (0..population_size)
        .into_par_iter()
        // .zip(rngs.iter_mut())
        .zip(rngs.par_iter_mut())
        .map(|(_, mut rng)| {
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
    population.extend_from_within(..);

    let mut children: Vec<Agent> = Vec::with_capacity(population_size);

    let mut iteration = 0;
    let mut f_eps = population[0].fitness - population[population_size - 1].fitness;
    while iteration < max_iter && f_eps > f_eps_min {
        if population.iter().any(|a| a.fitness.is_nan()) {
            return Err(Error::NanInTargetFunction);
        }
        let distances = population
            .iter()
            .take(population_size)
            .map(|agent| agent.location.norm_l2())
            .enumerate();

        let partner_choice = distances
            .clone()
            .zip(rngs.iter_mut())
            .map(|((i, d1), mut rng)| {
                WeightedIndex::new(distances.clone().map(|(j, d2)| distance_func(i, j, d1, d2)))
                    .and_then(|s| Ok(&population[s.sample(&mut rng)]))
                    .or(Err(Error::WeightsAreZero))
            })
            .collect::<Result<Vec<&Agent>, Error>>()?;

        // children =
        population
            // .iter()
            .par_iter()
            .zip(partner_choice)
            // .zip(rngs.iter_mut())
            .zip(rngs.par_iter_mut())
            .map(|((p1, p2), mut rng)| {
                make_child(
                    &population[0],
                    learn_rate,
                    p1,
                    p2,
                    &mut rng,
                    sampling_bounds.clone(),
                    mutation_probability,
                    mutation_scale,
                    &f,
                )
            })
            // .collect();
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
    rng: &mut impl RngCore,
    sampling_bounds: impl Iterator<Item = Uniform<f64>>,
    mutation_probability: f64,
    mutation_scale: f64,
    f: impl Fn(&ArrayView1<f64>) -> f64,
) -> Agent {
    let w1: f64 = rng.gen();
    let location_unmutated = &(&p1.location * w1) + &(&p2.location * (1. - w1));

    let location_unlearned = if rng.gen::<f64>() < mutation_probability {
        let mutation: Array1<f64> = sampling_bounds.map(|s| s.sample(rng)).collect();
        &location_unmutated + &(&mutation - &location_unmutated) * mutation_scale
    } else {
        location_unmutated
    };
    let location = &location_unlearned + (&p0.location - &location_unlearned) * learn_rate;
    let fitness = f(&location.view());
    Agent { location, fitness }
}

#[inline(always)]
fn distance_func(i: usize, j: usize, d1: f64, d2: f64) -> f64 {
    if i == j {
        0.
    } else {
        (-(d1.powi(2) + d2.powi(2) - 2f64 * d1 * d2)).exp()
    }
}

#[derive(Debug)]
pub struct EvoResult {
    pub agent: Agent,
    pub n_iter: usize,
    pub f_eps: f64,
}

#[derive(Debug, Clone)]
pub struct Agent {
    pub location: Array1<f64>,
    pub fitness: f64,
}
