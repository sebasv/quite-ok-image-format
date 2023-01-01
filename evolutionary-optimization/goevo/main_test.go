package main

import (
	"math"
	"testing"
)

// Go does not enforce formatting to the same level that Rust does, which is a missed opportunity.
// Eg I'm a bit sloppy with the camelCase/Snake_case and its all fine
// Also I made a ridiculously long line and it wasn't reformatted
func TestEncodeEmpty(t *testing.T) {

	var levi = func(x []float64) float64 {
		var val = math.Pow(math.Sin(x[0]*3*math.Pi), 2) + math.Pow(x[0]-1, 2)*(1+math.Pow(math.Sin(x[1]*3*math.Pi), 2)) + math.Pow(x[1]-1, 2)*(1+math.Pow(math.Sin(x[1]*2*math.Pi), 2))
		return -val
	}
	var bounds = [][2]float64{{-10, 10}, {-10, 10}}
	var population_size = 100
	var max_iter = 100
	var learn_rate = 0.01
	var f_eps_min = 1e-12
	var mutation_probability = 0.1
	var mutation_scale = 0.1
	var out = evolution(levi, bounds, population_size, max_iter, f_eps_min, learn_rate, mutation_probability, mutation_scale)
	t.Logf("Got out: %+v\n", out)
}

func BenchmarkEvolution(b *testing.B) {

	var levi = func(x []float64) float64 {
		var val = math.Pow(math.Sin(x[0]*3*math.Pi), 2) + math.Pow(x[0]-1, 2)*(1+math.Pow(math.Sin(x[1]*3*math.Pi), 2)) + math.Pow(x[1]-1, 2)*(1+math.Pow(math.Sin(x[1]*2*math.Pi), 2))
		return -val
	}
	var bounds = [][2]float64{{-10, 10}, {-10, 10}}
	var population_size = 100
	var max_iter = 100
	var learn_rate = 0.01
	var f_eps_min = 1e-12
	var mutation_probability = 0.1
	var mutation_scale = 0.1
	for i := 0; i < b.N; i++ {
		evolution(levi, bounds, population_size, max_iter, f_eps_min, learn_rate, mutation_probability, mutation_scale)
	}
}
