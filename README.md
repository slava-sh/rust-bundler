# rust-bundler
Bundler creates a single-source-file version of a Cargo package.

[![Build status](https://travis-ci.org/slava-sh/rust-bundler.svg?branch=master)](https://travis-ci.org/slava-sh/rust-bundler)
[![Coverage report](https://codecov.io/gh/slava-sh/rust-bundler/branch/master/graph/badge.svg)](https://codecov.io/gh/slava-sh/rust-bundler)
[![Crates.io](https://img.shields.io/crates/v/bundler.svg)](https://crates.io/crates/bundler)

## Installation

```sh
cargo install bundler
```

## Usage

```sh
bundle path/to/project >output.rs
```

## Library Usage

```rust
extern crate bundler;

fn main() {
    let code = bundler::bundle("path/to/project");
    println!("{}", code);
}
```

## Similar Projects

* [lpenz/rust-sourcebundler](https://github.com/lpenz/rust-sourcebundler)
  is based on regular expressions, whereas this project manipulates the syntax tree
* [golang.org/x/tools/cmd/bundle](https://godoc.org/golang.org/x/tools/cmd/bundle) for Go
