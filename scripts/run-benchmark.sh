#!/usr/bin/env bash

echo "Running on $RAYON_NUM_THREADS cores"

cargo build --release

run() {
  echo "Running provider '$2' on entry collection '$1'..."
  
  ./target/release/aonav benchmark \
    --parallel \
    --replicates 3 \
    --minimal \
    --timeout 30 \
    --providers "$2" \
    "benchmark/entries/$1" \
    > "benchmark/results/NON_INCR-$1-$2.csv"

  ./target/release/aonav benchmark \
    --parallel \
    --replicates 3 \
    --minimal \
    --timeout 30 \
    --providers "$2" \
    --incremental \
    "benchmark/entries/$1" \
    > "benchmark/results/INCR-$1-$2.csv"

  echo "Done!"
}

for path in benchmark/entries/*/; do
  collection=$(basename $path)

  run "$collection" "AlphabeticalUnsound"
  run "$collection" "AlphabeticalComplete"
  run "$collection" "AlphabeticalRelevant"

  run "$collection" "MaxInfoGainRelevant"
  run "$collection" "MaxInfoGain"

  run "$collection" "SufficiencySeekerRelevant"
  run "$collection" "SufficiencySeeker"
done
