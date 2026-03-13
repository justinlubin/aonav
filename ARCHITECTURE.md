# Architecture

This file documents the high-level architecture of this repository. Further
documentation can be found in the code modules themselves.

At a high level, there are two main components of this repository:

- The main implementation of `aonav` (in `src/`)
- The benchmark we evaluated `aonav` on (in `benchmark/`)

The following sections provide additional details about these sections.

Briefly, though, the main other top-level files in this repository are:
- `artifact-eval/` contains materials for the Artifact Evaluation submission
- `examples/` contains additional examples that `aonav` can run on
- `scripts/` contains additional miscellaneous scripts (and contains an
explanatory `README.md` within)
- `Dockerfile` is the Dockerfile for the Artifact Evaluation
- `Makefile` contains directives for how to build the project and Docker image

## The main implementation of `aonav` (`src/`)

Our implementation of `aonav` depends on the
[aograph](https://crates.io/crates/aograph) and
[pbn](https://crates.io/crates/pbn) Rust crates. In the course of making
`aonav`, we developed the former crate as an open-source Rust library that
supports what we call the AND-OR Json Graph Specification so that others can
build tooling that works with AND-OR graphs. The latter crate provides the
abstractions for defining a Programming by Navigation synthesizer.

### The `partition_navigation/` subdirectory

**The `partition_navigation/` subdirectory is the most important part of this
repository.**

A good place to start to understand the codebase is the
`partition_navigation_core.rs` file in the `partition_navigation/` subdirectory.
This file instantiates Programming by Navigation for the AND-OR graph
modifications we define in our paper. For example, you can see how we define a
"step" and an "expression", including how we define a "partition" of an AND-OR
graph.

Within the same directory, the `providers.rs` file defines the step providers we
define in Section 5 and Appendix B of our paper.

The `oracle.rs` file defines a nonempty-completion oracle for our notion of
expressions that matches Appendix A of our paper.

Finally, the `generate.rs` file randomly generates solutions to synthesis
problems. We used this file to generate the solutions for our benchmark suite.

### The other files

- `ao_adapters.rs`: glue code for importing other AND-OR graph-like formats
- `benchmark.rs`: code to benchmark step providers (generates a CSV)
- `drivers.rs`: code to drive Programming by Navigation step providers (we use
the "solution-driven" driver for our benchmarking)
- `lib.rs`: Rust boilerplate
- `main.rs`: program entry point; defines acceptable CLI arguments and
dispatches to `main_handler.rs`
- `main_handler.rs`: top-level functions to interact with `aonav` functionality
- `menu.rs`: defines a "menu" of options that CLI users can choose between for
step providers
- `min_ones.rs`: finds SAT assignments with a minimal number of literals set to
true
- `model_count.rs`: counts models of SAT formulas
(uses [Ganak](https://github.com/meelgroup/ganak))
- `partition_navigation.rs`: Rust boilerplate
- `util.rs`: defines auxiliary utilities (timer, cartesian product, graph
randomization)

## The `aonav` benchmarks (`benchmark/`)

The `benchmark/` folder contains five further subdirectories. In descending
order of importance:

- `entries/` contains the actual benchmark entries we evaluated `aonav` on
(further subdivided by whether or not the entries were "reduced" with unit
propagation first or not)
- `analysis/` contains the code to analyze the evaluation results; its
subdirectory `out/` is where the analysis script deposits generated plots)
- `extra-entries/` and `originals/` contain some additional benchmark entries
that we did NOT evaluate `aonav` on; the former contains unreduced entries for
Argus, and the latter contains the "originals" of the benchmark entries (i.e.,
non-randomized, and without solutions).
- `results/` contains the results of benchmarking (i.e., this is where the
benchmarking harness deposits its results)

The benchmark can be run using the `artifact-eval.sh` or `run-benchmark.sh`
scripts in the top-level `scripts/` directory.

To generate the plots, `cd` into the `analysis`  directory and run
`uv run main.py`.
