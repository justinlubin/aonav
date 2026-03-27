
#!/usr/bin/env bash

mkdir -p AESOP
rm -rf AESOP
mkdir -p AESOP

cd originals
cargo b

for f in *.json; do
    case "$f" in
        aesop*)
            ../../target/debug/aonav convert -f AOJsonGraph --reduce --randomize "$f" > "../AESOP/$f"
            # Failed (goal node already true)
            if [ -s "../AESOP/$f" ]; then
                :
            else
                echo "failed: $f"
                rm "../AESOP/$f"
            fi
            ;;
    esac
done

cd ..

../target/debug/aonav generate-solutions -c 3 --parallel AESOP
