This is a Rust port of the [DeBroglie](https://github.com/BorisTheBrave/DeBroglie) library done for education and personal fulfillment. 

This project is nowhere near ready for any kind of real use. Right now the code is written to match the original C# code as closely as possible, including the insane reference knots C# lets you get away with.    

Initially, I used this project as a test for using ChatGPT to convert C# code into Rust. I expected it to fail miserably and take me far more time fixing than if I had just done the conversion myself. This is *exactly* what happened, but, along the way, I learned far more about Rust than if I had just started from scratch.

The roadmap is going to be like this:
- [x] Get the smallest possible vertical slice (small `GridTopology`) working with its output exactly matching the original, and with the internals being as close to the original as possible.
- [ ] Add a few more simple examples if possible and get them matching the original.
- [ ] Try to get images working
- [ ] Completely refactor the code to adhere to Rust's idioms. This means studying a lot of respected Rust libraries and seeing how they do things.
- [ ] TBD

## Testing
Unit test coverage at this point is not anywhere close to where it needs to be. At the moment, I am only unit testing critical pieces that are either not directly tested by integration tests, or to increase visibility on certain parts.

In general, though, I will not do comprehensive unit test coverage until I am at the place where I can refactor this code into more idiomatic Rust.
