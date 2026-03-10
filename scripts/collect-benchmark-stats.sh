#!/usr/bin/env bash

cargo b --release

for path in benchmark/entries/*/; do
  bn=$(basename "$path")
  ./target/release/aonav benchmark-stats "$path" > "benchmark/entries/${bn}.csv"
done
