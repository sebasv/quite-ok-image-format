package main

import (
	"fmt"
	"math"
	"math/rand"
	"sort"
)

type Agent struct {
	location []float64
	fitness  float64
}

func make_agent(bounds [][2]float64, f func([]float64) float64) Agent {
	var location = make([]float64, len(bounds))

	for i := 0; i < len(bounds); i++ {
		var min, max = bounds[i][0], bounds[i][1]
		location[i] = min + rand.Float64()*(max-min)
	}
	var fitness = f(location)
	return Agent{location: location, fitness: fitness}

}

func evolution(f func([]float64) float64, bounds [][2]float64, population_size int, max_iter int, f_eps_min, learn_rate, mutation_probability, mutation_scale float64) Result {
	var population = make([]Agent, 0, population_size*2)
	var agent_channel = make(chan Agent)
	var channel_make_agent = func(bounds [][2]float64, f func([]float64) float64, agent_channel chan Agent) {
		agent_channel <- make_agent(bounds, f)
	}
	for i := 0; i < int(population_size)*2; i++ {
		go channel_make_agent(bounds, f, agent_channel)
	}
	for len(population) < int(population_size)*2 {
		population = append(population, <-agent_channel)
	}
	sort.Slice(population, func(i int, j int) bool { return population[i].fitness > population[j].fitness })
	var distances = make([]float64, population_size)
	var distance_matrix = make([]float64, population_size*population_size)
	var mate_selection = make([]int, population_size)

	var f_eps = population[0].fitness - population[population_size-1].fitness
	var iter = 0
	for iter < max_iter && f_eps > f_eps_min {

		fill_distances_and_distance_matrix(population_size, &distances, &population, &distance_matrix)

		// find correct index
		select_mates(population_size, &distance_matrix, &mate_selection)
		for i := 0; i < population_size; i++ {
			go make_child(population, i, mate_selection[i], mutation_probability, mutation_scale, bounds, learn_rate, f, agent_channel)
		}
		for i := 0; i < population_size; i++ {
			population[population_size+i] = <-agent_channel
		}
		sort.Slice(population, func(i int, j int) bool { return population[i].fitness > population[j].fitness })

		iter++
		f_eps = population[0].fitness - population[population_size-1].fitness
	}
	return Result{agent: population[0], f_eps: f_eps, iter: iter}
}

type Result struct {
	agent Agent
	f_eps float64
	iter  int
}

func make_child(population []Agent, p1 int, p2 int, mutation_probability float64, mutation_scale float64, bounds [][2]float64, learn_rate float64, f func([]float64) float64, wait_channel chan Agent) {
	var location = make([]float64, len(bounds))
	var w1 = rand.Float64()
	var w2 = 1 - w1
	var will_mutate = rand.Float64() < mutation_probability
	for i := 0; i < len(population[p1].location); i++ {
		location[i] = population[p1].location[i]*w1 + population[p2].location[i]*w2
		if will_mutate {
			var min, max = bounds[i][0], bounds[i][1]
			var mutation = min + (max-min)*rand.Float64()
			location[i] += mutation_scale * (mutation - location[i])
		}
		location[i] += learn_rate * (population[0].location[i] - location[i])
	}
	var fitness = f(location)
	wait_channel <- Agent{location: location, fitness: fitness}
}

func select_mates(population_size int, distance_matrix *[]float64, mate_selection *[]int) {

	for i_mate := 0; i_mate < population_size; i_mate++ {
		var i = i_mate * population_size
		var choice_w = rand.Float64() * (*distance_matrix)[i+population_size-1]
		var ix = 0
		for (*distance_matrix)[i+ix] < choice_w {
			ix++
		}
		(*mate_selection)[i_mate] = ix
	}
}

func fill_distances_and_distance_matrix(population_size int, distances *[]float64, population *[]Agent, distance_matrix *[]float64) {
	for i := 0; i < population_size; i++ {
		(*distances)[i] = 0
		for _, v := range (*population)[i].location {
			(*distances)[i] += v * v
		}
		(*distances)[i] = math.Sqrt((*distances)[i])
	}
	for i := 0; i < population_size; i += population_size {
		for j := 0; j < population_size; j++ {
			var ix = i + j
			if i == j {
				(*distance_matrix)[ix] = 0
			} else {
				(*distance_matrix)[ix] = math.Exp(-(math.Pow((*distances)[i], 2) + math.Pow((*distances)[j], 2) - 2*(*distances)[i]*(*distances)[j]))
			}
			if j > 0 {
				// make it cumulative
				(*distance_matrix)[ix] += (*distance_matrix)[ix-1]
			}
		}
	}
}

func main() {
	fmt.Println("hello world!")
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
	fmt.Printf("Got out: %+v\n", out)
}
