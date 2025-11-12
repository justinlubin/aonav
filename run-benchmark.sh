#!/usr/bin/env bash

cargo b
./target/debug/under benchmark -p Re,AU,AC --parallel benchmark > results/prelim.csv
