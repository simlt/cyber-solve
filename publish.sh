#!/bin/bash
set -e

# Cleanup tmp
if [[ -d ./tmp ]]; then
    rm -rf ./tmp/*
fi

mkdir -p ./tmp/cyber-solve

# build 
cargo build --release

# prepare files
cp assets config target/release/cyber-solve.exe tmp/cyber-solve/

mkdir -p dist/
zip -r dist/cyber-solve-release.zip tmp/cyber-solve
