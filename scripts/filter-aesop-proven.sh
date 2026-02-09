# used to move not proven examples out of the aesop-examples folder and into a new one
cd ..
cd examples
mkdir aesop-not-proven

for file in aesop-examples/*; do
    if [[ "${file:0:19}" == "aesop-examples/not_" ]]; then
        mv $file aesop-not-proven
    fi
done