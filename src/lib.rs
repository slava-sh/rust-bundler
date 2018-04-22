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

pub fn bundle(project: &Path) -> String {
    let cargo_toml = read_file(&project.join("Cargo.toml")).expect("failed to read Cargo.toml");
    let cargo_toml: toml::Value = toml::from_str(&cargo_toml).expect("failed to parse Cargo.toml");
    let package_name = cargo_toml
        .get("package")
        .expect("Cargo.toml has no package")
        .get("name")
        .expect("Cargo.toml has no package.name")
        .as_str()
        .expect("failed to read package name");
    let src = project.join("src");
    let code = read_file(&src.join("main.rs")).expect("failed to read main.rs");
    let mut file = syn::parse_file(&code).expect("failed to parse main.rs");
    Expander {
        base_path: &src,
        crate_name: package_name,
    }.visit_file_mut(&mut file);
    let code = file.into_tokens().to_string();
    prettify(code)
}

struct Expander<'a> {
    base_path: &'a Path,
    crate_name: &'a str,
}

impl<'a> Expander<'a> {
    fn expand_extern_crate(&self, file: &mut syn::File) {
        let mut new_items = vec![];
        for item in file.items.drain(..) {
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
        file.items = new_items;
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

    fn remove_crate_path(&mut self, path: &mut syn::Path) {
        if starts_with(path, self.crate_name) {
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
        self.expand_extern_crate(file);
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
        self.remove_crate_path(path);
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

fn starts_with(path: &syn::Path, segment: &str) -> bool {
    if let Some(el) = path.segments.first() {
        if el.value().ident.to_string() == segment {
            return true;
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
