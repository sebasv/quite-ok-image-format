# The Quite-OK-Image format

This is a toy implementation of the QOI format to compare my personal process of banging out a minimal solution to a nontrivial problem in Rust and Go. E.g. I was not very strict in things like error handling and input validation.

I went into this project with some tens of hours of experience in Rust and <30 minutes of experience in Go.

It is definitely true what they say: Getting productive is easy in Go. I did the Rust implementation first, then the Go implementation. Both took about the same time, so my experience in Rust was worth about the same time as it took me to get familiar with the algorithm:

```
 Rust |--- learning the algo -->|--- coding in Rust -->|
 Go   |--------- learning Go and coding in Go -------->|
 ```

Code-style wise my focus was on getting shit done, so both solutions don't deserve a prize and are at best mildly idiomatic for the respective languages. 
* In Rust I am always drawn towards iterators and handling state by matching Enums. 
* In Go it felt not quite natural to do a `range` so I went for indexed iteration.

The most striking difference between both experiences is that I wrote the Rust impl, wrote tests for it, all passed on the first attempt and that was it. After writing the Go impl, even though that was my second time implementing the algorithm, the majority of tests failed. My errors were of the following types:
* off-by-one in the indexing
* field orders flipped in struct creation
* handling edge cases.

### Edit
After adding a more extensive test I did find a bug in my Rust code! I forgot to update the `previous_pixel` in the decoder after an index hit.

## Rust vs Go

* In Rust, cargo will take care of all the boilerplate. By comparison the available `go` commands feel a bit lacking. That said, the `go` commands are still a step up from any other tooling I have experience with.
* The Go integration in VSCode is slightly better than the Rust integratino. Specifically the test suite functionality of VSCode works better for Go. The Go formatter will add an explicit cast sometimes when I add a `uint` to an `int`, whereas the Rust formatter will just tell you to fix it.
* In Rust I know what happens with my structs. I can tell by looking at a statement if a copy is created, and I can trust the optimizer to reuse memory if for example a copy gets reassigned to a mutable variable. In Go I had no idea if an object is copied until I tried. You have to look up blog posts to learn that Go copies on assingment.
* Go does not do immutable, so there are no `const` structs or arrays. I missed them, and it led me to hardcode some things.
* I don't like that everything is heaped in Go. It means that one optimization you can always do on Go code is not use structs, which is will not help readability.
* Go code is shorter.
* The Rust formatter is much more strict. The Go formatter leaves plenty of room for flamewars about indentation and camel vs snake naming.
* The Go error convention is identical to Rust's (in that you return a value), except that Rust uses a proper Monad which provides all kinds of conveniences. In contrast Go requires you to write a LOT of if statements.
* I did not get to try Go's prime selling point: Goroutines. They look really nice and I'd like to experiment with them on a suitable problem.
* The Go code was a bit (~20%) slower for the encoder, but significantly faster (12x) for the decoder. I made 2 mistakes in the Rust code that are less bad in garbage-collected languages:
  - I created (heap-allocated) `Vec`s as intermediaries in a single computation. Such allocations are relatively cheap in a garbage-collected language like Go due to the reuse of free memory but very expensive in Rust due to its explicit memory management; each of these `Vec`s has to be allocated and freed, and the compiler failed to optimize them out of my code.
  - I filled a pre-allocated but not pre-initialized `Vec` (`Vec::with_capacity`) that I filled with `push`es, which involves a lot of costly boundary checks etc. Again in simple code Rust would have optimized this away, but not this time. The solution was to do a slightly redundant `resize` on my vec to initialize it, and using pattern matching on a slice into the vec to avoid checks.
  Overall the code is still idiomatic Rust, and now the decode is ~35% faster than the Go version.