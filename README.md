# `aonav`

`aonav` is a tool to help debug failing proofs over an AND-OR graph. It works by
letting the user interactively navigate to AND-OR graph modifications using
[Programming by Navigation](https://doi.org/10.1145/3729264).

## Running `aonav`

Simply run `cargo r -- interact FILENAME`. For example, try out
`cargo r -- interact examples/bio.json`!

## Dependencies

To run the basic version of `aonav`, all you need is [Rust](https://rustup.rs/)!

### Advanced dependencies

- For the MaxInfoGain strategy: [Ganak v2.5.3](https://github.com/meelgroup/ganak/releases/tag/release%2F2.5.3)
- For running the evaluation: [uv](https://docs.astral.sh/uv/getting-started/installation/)
- For building the Docker image for the artifact evaluation: [podman](https://podman.io)

## About the codebase

For more information about the codebase, dive into [ARCHITECTURE.md](ARCHITECTURE.md)!
