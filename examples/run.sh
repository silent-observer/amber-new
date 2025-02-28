#!/bin/bash
cd $(dirname $0)
CMD=${1:-cargo run}

for d in */ ; do
    pushd $d
    $CMD test *.lua
    popd
done