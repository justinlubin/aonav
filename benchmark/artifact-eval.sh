if [ -z "$1" ]; then
  RAYON_NUM_THREADS="$(nproc)"
else
  RAYON_NUM_THREADS="$1"
fi

echo "Running on ${RAYON_NUM_THREADS} cores (implementations are single-threaded)"

for path in benchmark/entries/*; do
  collection=$(basename $path)

  echo "Benchmarking '${collection}'..."

  aonav benchmark \
    --parallel \
    --replicates 3 \
    --minimal \
    --providers AlphabeticalUnsound,AlphabeticalComplete,AlphabeticalRelevant \
    "${path}" \
    > "benchmark/results/${collection}.csv"
done
