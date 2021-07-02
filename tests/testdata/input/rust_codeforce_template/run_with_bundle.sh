#!/usr/bin/env bash

set -e

if [[ ! -f Cargo.toml ]]; then
  echo "Not a cargo project. Aborting"
  exit 1
fi

bundle . > /dev/shm/output.rs
rustc /dev/shm/output.rs -o /dev/shm/output
/dev/shm/output