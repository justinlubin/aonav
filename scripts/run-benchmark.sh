#!/usr/bin/env bash

cargo build

for suite in $(ls benchmark/suites); do
  ./target/debug/under benchmark \
    --parallel \
    --replicates 3 \
    --minimal \
    --providers AlphabeticalUnsound,AlphabeticalComplete,AlphabeticalRelevant \
    "benchmark/suites/${suite}" \
    > "benchmark/results/${suite}.csv"
done
