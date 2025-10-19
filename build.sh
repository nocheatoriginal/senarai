#!/bin/bash

if [ ! -d "target/release" ] || [ "$1" == "--clean" ] || [ "$1" == "-c" ]; then
    echo "Building release version..."
    cargo build --release
    if [ $? -ne 0 ]; then
        echo "Build failed." >&2
        exit 1
    fi
fi

cp config.yaml target/release
./target/release/senarai
