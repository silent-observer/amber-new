#!/bin/bash
cd $(dirname $0)
CMD=${1:-cargo run}

for d in */ ; do
    pushd $d
    if [ -f test*.lua ]; then
        $CMD test test*.lua
    fi
    popd
done