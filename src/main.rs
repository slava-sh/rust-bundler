extern crate syn;
extern crate quote;

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{PathBuf, Path};
use std::process;

use quote::ToTokens;
use syn::visit_mut::VisitMut;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: expand-mods path/to/main.rs");
        process::exit(1);
    }
    let path = Path::new(&args[1]);
    let code = read_file(path).expect("failed to read file");
    let mut syntax = syn::parse_file(&code).expect("failed to parse file");
    expand_mods(
        &mut syntax,
        path.parent().expect("no parent dir").to_owned(),
    );
    println!("{}", &syntax.into_tokens().to_string());
}

fn expand_mods(syntax: &mut syn::File, base_path: PathBuf) {
    ExpandingVisitor { base_path: base_path }.visit_file_mut(syntax);
}

struct ExpandingVisitor {
    base_path: PathBuf,
}

impl VisitMut for ExpandingVisitor {
    fn visit_item_mod_mut(&mut self, item: &mut syn::ItemMod) {
        if let Some(ref mut it) = item.content {
            for it in &mut (it).1 {
                self.visit_item_mut(it);
            }
        } else {
            let name = item.ident.to_string();
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
            expand_mods(&mut syntax, base_path);
            item.content = Some((Default::default(), syntax.items));
        }
    }
}

fn read_file(path: &Path) -> Option<String> {
    let mut buf = String::new();
    File::open(path).ok()?.read_to_string(&mut buf).ok()?;
    Some(buf)
}
