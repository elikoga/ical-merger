#!/usr/bin/env bash

set -e

# generate example.config.yaml with ./generate_config_example.py

./generate_config_example.py > example.config.yaml

# also run tests, clippy and fmt

cargo test --all --locked

cargo clippy

cargo fmt --all -- --check