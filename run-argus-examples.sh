
for file in "all-argus"/*; do
    out_file_0=${file%?????} 
    out_file_1=${out_file_0:14}
    out_file="all-argus-ao/$out_file_1.json"
    
    ./target/debug/under convert --format Argus $file > $out_file
    ./target/debug/under render $out_file
    mv out/RENDERED.pdf out/$out_file_1.pdf
    
done
