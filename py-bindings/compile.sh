#! /usr/bin/bash

PWD=$(git rev-parse --show-toplevel)/py-bindings
cd $PWD
cargo run --bin stub_gen
maturin develop