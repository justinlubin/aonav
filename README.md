# `aonav`

`aonav` is a tool to help debug failing proofs over an AND-OR graph. It works by
letting the user interactively navigate to AND-OR graph modifications using
[Programming by Navigation](https://doi.org/10.1145/3729264).

## Running `aonav`

Simply run `cargo r -- interact FILENAME`. For example, try out
`cargo r -- interact examples/bio.json`!

You can try out a different step provider, say `AlphabeticalRelevant`, like so:
`cargo r -- interact -p AlphabeticalRelevant examples/bio.json`. For a full list
of step providers, please see the `Provider` enum in [src/menu.rs](src/menu.rs).

For more help, run `cargo r -- help`.

## Dependencies

To run the basic version of `aonav`, all you need is [Rust](https://rustup.rs/)!

### Advanced dependencies

- For the MaxInfoGain strategy: [Ganak v2.5.3](https://github.com/meelgroup/ganak/releases/tag/release%2F2.5.3)
- For running the evaluation: [uv](https://docs.astral.sh/uv/getting-started/installation/)
- For building the Docker image for the artifact evaluation: [podman](https://podman.io)

## About the codebase

For more information about the codebase, dive into [ARCHITECTURE.md](ARCHITECTURE.md)!

# The AND-OR JSON Graph Format v1

`aonav` works with a standardized file format for AND-OR graphs that we call the
_AND-OR JSON Graph Format (AO-JGF) v1_, which is a refinement of the
[JSON Graph Format (JGF) v2](https://jsongraphformat.info/).
It is a refinement in the sense that any tooling that works with the JGF (and,
consequently, any tooling that works with JSON) should work with the AO-JGF
format.

```
{
  "graph": {
    "nodes": { <NODE_ID>: <NODE_VAL> },
    "edges": [<EDGE>],
    "metadata": { "goal": <NODE_ID> }
  }
}
```

### The "nodes" field

The "nodes" field is an object whose keys we call _node identifiers_ and whose
values are objects of the following form:

```
{ "label": <OPTIONAL STRING>, "metadata": { "kind": <KIND> }}
```

The "label" field is an optional string for display purposes only (there is no
uniqueness requirement).

The "kind" metadata field must be either "AND" (for AND nodes) or "OR" (for OR
nodes).

### The "edges" field

The "edges" field is a list of objects of the following form:

```
{ "source": <NODE_ID>, "target": <NODE_ID> }
```

The node identifiers must be present as keys in the "nodes" field. Additionally
the kind of the source must be different from the kind of the target (AND-OR
graphs are bipartite).

An edge from a node `A` to a node `f` means that `A` _depends on_ `f` (i.e.,
that `f` is a subgoal of `A`). Thus, it is likely that the "goal" node (see
below) will have only _outgoing_ edges.

### The "goal" field

The "goal" subfield of the "metadata" field must be a node identifier that is
present in the keys of the "nodes" field, and the kind of the node must be "OR".

### Example AO-JGF file

```
{
  "graph": {
    "nodes": {
      "A": {
        "metadata": {
          "kind": "OR"
        }
      },
      "B": {
        "metadata": {
          "kind": "OR"
        }
      },
      "C": {
        "metadata": {
          "kind": "OR"
        }
      },
      "f": {
        "metadata": {
          "kind": "AND"
        }
      }
    },
    "edges": [
      {
        "source": "A",
        "target": "f"
      },
      {
        "source": "f",
        "target": "B"
      },
      {
        "source": "f",
        "target": "C"
      }
    ],
    "metadata": {
      "goal": "A"
    }
  }
}
```
