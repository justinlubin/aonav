mkdir -p and_or_filter_unproven/proven
mkdir -p and_or_filter_unproven/not_proven

for path in benchmark/aesop-traces/raw-traces/*.txt; do
  python3 scripts/aesop-scripts/make_tree.py $path
done
