mkdir -p out/figures

cargo b

for f in "overview" "overview-example-messy1" "path-not-provable" "shared" "shared-1" "shared-2" "aesop-overview-example"; do
  ./target/debug/aonav render \
    examples/$f.json > out/figures/temp.pdf
  gs \
    -sDEVICE=pdfwrite \
    -o out/figures/$f.pdf \
    -c "<< /AlwaysOutline [/] >> setdistillerparams" \
    -f out/figures/temp.pdf
  rm out/figures/temp.pdf
done
