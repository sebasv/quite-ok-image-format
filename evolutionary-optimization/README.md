# Evolutionary optimization in Rust and Go

Two implementations, both employing paralellization over generating the next generation.

Much like with the QOI encoder/decoder, writing Go was "easy" and writing Rust was "hard". By that I mean that Go felt like it required less thought, except that the price for this was incidentally sharing memory between threads, and not having a thread-local random number generator.
Rust forced me to think about these concepts ahead of compilation.

In addition I learned that Go is not purely 'pass-by-value': If you pass an array or slice into a function, Go will pass a fresh pointer to the same memory, which I would consider pass-by-reference. In other words, a Go function can have side effect such as mutation of elements in a passed slice or array, even if it were passed by value. You cannot change the length of the array though, for that you would need to pass a "pointer to the array", which technically would thus be "a pointer to the pointer to the memory". Now that leaves me with this question:

```go

// pass `a` by value; `foo::a` shares its memory with `main::arr` until the `append`
func foo(a []byte) {
    // change `a` in-place, affects `main::arr`
    a[0] = 255
    // this implicitly copies the contents into a new array, but **ONLY** if the capacity of arr is insufficient?!
    a = append(a, 0)
    a[1] = 255
}
func main() {
    var arr = []byte{0,1,2}
    foo(arr)
}

```

Again, in Rust my first attempt had some intermediate data structures that slowed things down but after a minute of profiling I realized my mistake and I removed these structures, which is a breeze due to iterators.