# Underivability explorations

`cargo run` launches an interactive command line that can load a proof system
and apply various transformations to it. See `scripts/basic.txt` for an example
script that loads in the example `example/basic.txt` and does some basic
operations. Use the `help` command to see a list of all commands.

Whenever the proof system updates, it will be displayed to the terminal.
Additionally, the "proof system graph" will be saved to `out/proof_system.dot`
in the [dot](https://graphviz.org/docs/layouts/dot/) file format. The `Makefile`
will create a PDF file of the proof system graph from this format.

_**Tip:** You can type each command into the command line manually or you can run
`cat scripts/basic.txt | cargo run`. If you run the commands manually, you can
skip the `displaycommand` command, which echoes back the issued command
(helpful for non-interactive use, as with the piping above)._

## Repository structure

- `src`: source code for the application
- `examples`: example proof systems
- `scripts`: example scripts for the interactive command line
