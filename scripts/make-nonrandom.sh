cd benchmark/suites/nonrandom

for f in ../reduced/*.json; do
  ln -s $f .
done

for f in ../unreduced/manual-*.json; do
  ln -s $f .
done

for f in ../unreduced/aesop-*.json; do
  ln -s $f .
done
