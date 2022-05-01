#!/bin/bash
set -e

# Cleanup tmp
if [[ -d ./tmp ]]; then
    rm -rf ./tmp/*
fi

TMP_PATH='./tmp/cyber-solve'
mkdir -p $TMP_PATH

# build 
cargo build --release

# prepare asset and binary files
mkdir -p $TMP_PATH/assets/
cp -r assets/tesseract assets/images $TMP_PATH/assets/
cp -r config target/release/cyber-solve.exe $TMP_PATH/

# make zip archive
if [[ ! -d ./dist ]]; then
    mkdir -p ./dist/
fi

ARCHIVE_OUT_PATH='./dist/cyber-solve-release.zip'
zip -r $ARCHIVE_OUT_PATH tmp/cyber-solve
echo "Created archive $ARCHIVE_OUT_PATH"
