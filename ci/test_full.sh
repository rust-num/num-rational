#!/bin/bash

set -ex

echo Testing num-rational on rustc ${TRAVIS_RUST_VERSION}

STD_FEATURES="bigint-std serde"

case "$TRAVIS_RUST_VERSION" in
  1.3[1-5].*) NO_STD_FEATURES="serde" ;;
  *) NO_STD_FEATURES="bigint serde" ;;
esac


# num-rational should build and test everywhere.
cargo build --verbose
cargo test --verbose

# It should build with minimal features too.
cargo build --no-default-features
cargo test --no-default-features

# Each isolated feature should also work everywhere.
for feature in $STD_FEATURES; do
  cargo build --verbose --no-default-features --features="std $feature"
  cargo test --verbose --no-default-features --features="std $feature"
done

# test all supported features together
cargo build --features="std $STD_FEATURES"
cargo test --features="std $STD_FEATURES"

# Each no-std isolated feature should also work everywhere.
for feature in $NO_STD_FEATURES; do
  cargo build --verbose --no-default-features --features="$feature"
  cargo test --verbose --no-default-features --features="$feature"
done

# test all no-std supported features together
cargo build --no-default-features --features="$NO_STD_FEATURES"
cargo test --no-default-features --features="$NO_STD_FEATURES"
