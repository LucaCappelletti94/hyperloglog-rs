# Evaluation of composite hash
This small experiment answers the question: how often do composite hashes, i.e. the hashes used in Hybrid HLLs up until they fit in the registers, collide, given a precision and bit-size?

## Running the experiment
To run the experiment, simply run the following command:

```bash
RUSTFLAGS='-C target-cpu=native' cargo run --release
```

And it will write out the results to [`collision_rates.csv`]().