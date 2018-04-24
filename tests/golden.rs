extern crate bundler;
extern crate goldenfile;

use std::io::Write;
use std::fs;
use std::path::Path;

use goldenfile::Mint;

const INPUT_DIR: &'static str = "tests/testdata/input";
const OUTPUT_DIR: &'static str = "tests/testdata/output";

fn main() {
    let mut mint = Mint::new(OUTPUT_DIR);
    for entry in fs::read_dir(INPUT_DIR).expect("read_dir failed") {
        let input_path = entry.expect("no entry").path();
        let input_name = input_path.file_name().expect("no file name");
        let output_name = Path::new(input_name).with_extension("rs");
        let mut output_file = mint.new_goldenfile(output_name).expect(
            "new_goldenfile failed",
        );
        let output = bundler::bundle(&input_path).expect("bundle failed");
        write!(output_file, "{}", output).expect("write! failed");
    }
}
