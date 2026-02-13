#!/usr/bin/env bash

echo "Running on $RAYON_NUM_THREADS cores"

cargo build

run() {
  echo "Running provider '$2' on suite '$1'..."
  ./target/debug/under benchmark \
    --parallel \
    --replicates 3 \
    --minimal \
    --providers "$2" \
    "benchmark/suites/$1" \
    > "benchmark/results/$1-$2.csv"
  echo "Done!"
}

run "unreduced" "AlphabeticalUnsound"
run "unreduced" "AlphabeticalComplete"
run "unreduced" "AlphabeticalRelevant"

run "reduced" "AlphabeticalUnsound"
run "reduced" "AlphabeticalComplete"
run "reduced" "AlphabeticalRelevant"

run "nonrandom" "MaxInfoGain"
run "nonrandom" "MaxInfoGainRelevant"
