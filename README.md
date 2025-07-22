# Underivability explorations

`cargo run` launches an interactive command line that can load a proof system
and apply various transformations to it. See `scripts/basic.txt` for an example
script that loads in the example `example/basic.txt` and does some basic
operations. You can type each command into the command line manually or you
can run `cat scripts/basic.txt | cargo run`. If you run the commands manually,
you can skip the `displaycommand` command, which echoes back the issued command
(helpful for non-interactive use, as with the piping above).

- `src`: source code for the application
- `examples`: example proof systems
- `scripts`: example scripts for the interactive command line
