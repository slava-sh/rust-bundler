extern crate assert_cli;

use assert_cli::Assert;

#[test]
fn usage() {
    Assert::main_binary()
        .with_args(&[] as &[&str])
        .fails()
        .stderr()
        .contains("Usage: bundle")
        .unwrap();
}

#[test]
fn bundle_self() {
    Assert::main_binary()
        .with_args(&["."])
        .stdout()
        .contains("pub fn bundle<")
        .stdout()
        .contains("let code = bundle(")
        .unwrap();
}
