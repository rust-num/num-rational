#!/bin/bash

set -e

get_rust_version() {
  local array=($(rustc --version));
  echo "${array[1]}";
  return 0;
}

if [ -z ${TRAVIS+x} ]
then RUST_VERSION=$(get_rust_version)  # we're not in travis
else RUST_VERSION=$TRAVIS_RUST_VERSION  # we're in travis
fi

if [ -z ${RUST_VERSION} ]
then  echo "WARNING: RUST_VERSION is undefined or empty string" 1>&2
else  echo Testing num-rational on rustc ${RUST_VERSION}
fi

set -x

STD_FEATURES="bigint-std serde"

case "$RUST_VERSION" in
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
