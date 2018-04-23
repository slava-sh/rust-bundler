//! See [README.md](https://github.com/slava-sh/rust-bundler/blob/master/README.md)
extern crate quote;
extern crate rustfmt;
extern crate syn;
extern crate toml;

use std::fs::File;
use std::io::{Read, Sink};
use std::mem;
use std::path::Path;

use quote::ToTokens;
use syn::visit_mut::VisitMut;
use syn::punctuated::Punctuated;

/// Creates a single-source-file version of a Cargo package.
pub fn bundle<P: AsRef<Path>>(package_path: P) -> String {
    let package_path = package_path.as_ref();
    let cargo_toml =
        read_file(&package_path.join("Cargo.toml")).expect("failed to read Cargo.toml");
    let cargo_toml = toml::from_str(&cargo_toml).expect("failed to parse Cargo.toml");
    let crate_name = get_crate_name(cargo_toml).expect("cannot determine crate name");
    let src = package_path.join("src");
    let code = read_file(&src.join("main.rs")).expect("failed to read main.rs");
    let mut file = syn::parse_file(&code).expect("failed to parse main.rs");
    Expander {
        base_path: &src,
        crate_name: &crate_name,
    }.visit_file_mut(&mut file);
    let code = file.into_tokens().to_string();
    prettify(code)
}

fn get_crate_name(cargo_toml: toml::Value) -> Option<String> {
    cargo_toml
        .get("lib")
        .and_then(|lib| lib.get("name"))
        .map(|name| {
            name.as_str().expect("lib name is not a string").to_owned()
        })
        .or_else(|| {
            cargo_toml
                .get("package")
                .and_then(|package| {
                    package.get("name").map(|name| {
                        name.as_str().expect("package name is not a string")
                    })
                })
                .map(|name| name.replace("-", "_"))
        })
}

struct Expander<'a> {
    base_path: &'a Path,
    crate_name: &'a str,
}

impl<'a> Expander<'a> {
    fn expand_items(&self, items: &mut Vec<syn::Item>) {
        self.expand_extern_crate(items);
        self.expand_use_path(items);
    }

    fn expand_extern_crate(&self, items: &mut Vec<syn::Item>) {
        let mut new_items = vec![];
        for item in items.drain(..) {
            if is_extern_crate(&item, self.crate_name) {
                eprintln!(
                    "expanding crate {} in {}",
                    self.crate_name,
                    self.base_path.to_str().unwrap()
                );
                let code =
                    read_file(&self.base_path.join("lib.rs")).expect("failed to read lib.rs");
                let lib = syn::parse_file(&code).expect("failed to parse lib.rs");
                new_items.extend(lib.items);
            } else {
                new_items.push(item);
            }
        }
        *items = new_items;
    }

    fn expand_use_path(&self, items: &mut Vec<syn::Item>) {
        let mut new_items = vec![];
        for item in items.drain(..) {
            if !is_use_path(&item, self.crate_name) {
                new_items.push(item);
            }
        }
        *items = new_items;
    }

    fn expand_mods(&self, item: &mut syn::ItemMod) {
        if item.content.is_some() {
            return;
        }
        let name = item.ident.to_string();
        let other_base_path = self.base_path.join(&name);
        let (base_path, code) = vec![
            (self.base_path, format!("{}.rs", name)),
            (&other_base_path, String::from("mod.rs")),
        ].into_iter()
            .flat_map(|(base_path, file_name)| {
                read_file(&base_path.join(file_name)).map(|code| (base_path, code))
            })
            .next()
            .expect("mod not found");
        eprintln!("expanding mod {} in {}", name, base_path.to_str().unwrap());
        let mut file = syn::parse_file(&code).expect("failed to parse file");
        Expander {
            base_path,
            crate_name: self.crate_name,
        }.visit_file_mut(&mut file);
        item.content = Some((Default::default(), file.items));
    }

    fn expand_crate_path(&mut self, path: &mut syn::Path) {
        if path_starts_with(path, self.crate_name) {
            let new_segments = mem::replace(&mut path.segments, Punctuated::new())
                .into_pairs()
                .skip(1)
                .collect();
            path.segments = new_segments;
        }
    }
}

impl<'a> VisitMut for Expander<'a> {
    fn visit_file_mut(&mut self, file: &mut syn::File) {
        for it in &mut file.attrs {
            self.visit_attribute_mut(it)
        }
        self.expand_items(&mut file.items);
        for it in &mut file.items {
            self.visit_item_mut(it)
        }
    }

    fn visit_item_mod_mut(&mut self, item: &mut syn::ItemMod) {
        for it in &mut item.attrs {
            self.visit_attribute_mut(it)
        }
        self.visit_visibility_mut(&mut item.vis);
        self.visit_ident_mut(&mut item.ident);
        self.expand_mods(item);
        if let Some(ref mut it) = item.content {
            for it in &mut (it).1 {
                self.visit_item_mut(it);
            }
        }
    }

    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        self.expand_crate_path(path);
        for mut el in Punctuated::pairs_mut(&mut path.segments) {
            let it = el.value_mut();
            self.visit_path_segment_mut(it)
        }
    }
}

fn is_extern_crate(item: &syn::Item, crate_name: &str) -> bool {
    if let syn::Item::ExternCrate(ref item) = *item {
        if item.ident.to_string() == crate_name {
            return true;
        }
    }
    false
}

fn path_starts_with(path: &syn::Path, segment: &str) -> bool {
    if let Some(el) = path.segments.first() {
        if el.value().ident.to_string() == segment {
            return true;
        }
    }
    false
}

fn is_use_path(item: &syn::Item, first_segment: &str) -> bool {
    if let syn::Item::Use(ref item) = *item {
        if let syn::UseTree::Path(ref path) = item.tree {
            if path.ident.to_string() == first_segment {
                return true;
            }
        }
    }
    false
}

fn read_file(path: &Path) -> Option<String> {
    let mut buf = String::new();
    File::open(path).ok()?.read_to_string(&mut buf).ok()?;
    Some(buf)
}

fn prettify(code: String) -> String {
    let config = Default::default();
    let out: Option<&mut Sink> = None;
    let result = rustfmt::format_input(rustfmt::Input::Text(code), &config, out)
        .expect("rustfmt failed");
    let ref buf = result.1.first().expect("rustfmt returned no code").1;
    format!("{}", buf)
}
