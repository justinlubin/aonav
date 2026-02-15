#!/usr/bin/env bash

echo "Running on $RAYON_NUM_THREADS cores"

cargo build

run() {
  echo "Running provider '$2' on entry collection '$1'..."
  
  ./target/debug/under benchmark \
    --parallel \
    --replicates 3 \
    --minimal \
    --timeout 30 \
    --providers "$2" \
    "benchmark/entries/$1" \
    > "benchmark/results/$1-$2.csv"

  echo "Done!"
}

for path in benchmark/entries/*; do
  collection=$(basename $path)

  run "$collection" "AlphabeticalUnsound"
  run "$collection" "AlphabeticalComplete"
  run "$collection" "AlphabeticalRelevant"

  run "$collection" "MaxInfoGainRelevant"
  run "$collection" "MaxInfoGain"
done
