#!/usr/bin/env bash

cargo b
# ./target/debug/under benchmark -p Re,AU,AC -r 1 --parallel benchmark > results/prelim.csv
./target/debug/under benchmark -p AU,AC,AR -r 1 --parallel benchmark > results/prelim-relevant.csv
