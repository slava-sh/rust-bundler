extern crate syn;
extern crate quote;

use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{PathBuf, Path};
use std::process::{self, Command, Stdio};

use quote::ToTokens;
use syn::visit_mut::VisitMut;

fn main() {
    let mut args = env::args();
    let _ = args.next();
    let filename = match (args.next(), args.next()) {
        (Some(filename), None) => filename,
        _ => {
            eprintln!("Usage: expand-mods path/to/filename.rs");
            process::exit(1);
        }
    };

    let path = Path::new(&filename);
    let code = read_file(path).expect("failed to read file");
    let mut syntax = syn::parse_file(&code).expect("failed to parse file");
    expand_mods(
        &mut syntax,
        path.parent().expect("no parent dir").to_owned(),
    );
    println!("{}", format_code(&syntax.into_tokens().to_string()));
}

fn expand_mods(syntax: &mut syn::File, base_path: PathBuf) {
    ExpandingVisitor { base_path: base_path }.visit_file_mut(syntax);
}

struct ExpandingVisitor {
    base_path: PathBuf,
}

impl VisitMut for ExpandingVisitor {
    fn visit_item_mod_mut(&mut self, i: &mut syn::ItemMod) {
        if let Some(ref mut it) = i.content {
            for it in &mut (it).1 {
                self.visit_item_mut(it);
            }
        } else {
            let name = i.ident.to_string();
            let (base_path, code) = vec![
                (self.base_path.clone(), format!("{}.rs", name)),
                (self.base_path.join(&name), String::from("mod.rs")),
            ].into_iter()
                .flat_map(|(base_path, file_name)| {
                    read_file(&base_path.join(file_name)).map(|code| (base_path, code))
                })
                .next()
                .expect("mod not found");
            eprintln!("expanding mod {} in {}", name, base_path.to_str().unwrap());
            let mut syntax = syn::parse_file(&code).expect("failed to parse file");
            expand_mods(&mut syntax, base_path.to_owned());
            i.content = Some((Default::default(), syntax.items));
        }
    }
}

fn read_file(path: &Path) -> Option<String> {
    let mut buf = String::new();
    File::open(path).ok()?.read_to_string(&mut buf).ok()?;
    Some(buf)
}

fn format_code(code: &str) -> String {
    let mut child = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to execute rustfmt");
    child
        .stdin
        .as_mut()
        .expect("failed to get stdin")
        .write_all(code.as_bytes())
        .expect("failed to write to stdin");
    let output = child.wait_with_output().expect(
        "failed to wait for rustfmt",
    );
    String::from_utf8(output.stdout).expect("failed to parse rustfmt output")
}
