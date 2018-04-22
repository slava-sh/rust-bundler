extern crate quote;
extern crate rustfmt;
extern crate syn;
extern crate toml;

use std::fs::File;
use std::io::{Read, Sink};
use std::mem;
use std::path::Path;

use quote::ToTokens;

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
    expand(&mut file, package_name, &src);
    let code = file.into_tokens().to_string();
    prettify(code)
}

fn expand(file: &mut syn::File, package_name: &str, base_path: &Path) {
    expand_crate(file, base_path, package_name);
    expand_mods(file, base_path);
}

fn expand_crate(file: &mut syn::File, base_path: &Path, crate_name: &str) {
    let mut crate_item_i = None;
    for (i, item) in file.items.iter_mut().enumerate() {
        if let syn::Item::ExternCrate(ref mut item) = *item {
            if item.ident.to_string() == crate_name {
                crate_item_i = Some(i);
                break;
            }
        }
    }
    if let Some(i) = crate_item_i {
        eprintln!(
            "expanding crate {} in {}",
            crate_name,
            base_path.to_str().unwrap()
        );
        let code = read_file(&base_path.join("lib.rs")).expect("failed to read lib.rs");
        let mut lib = syn::parse_file(&code).expect("failed to parse lib.rs");
        expand_mods(&mut lib, base_path);
        let (crates, items) = lib.items.drain(..).partition(is_extern_crate);

        let item = mem::replace(&mut file.items[i], unsafe { mem::uninitialized() });
        if let syn::Item::ExternCrate(item) = item {
            file.items[i] = syn::Item::Mod(syn::ItemMod {
                attrs: item.attrs,
                vis: item.vis,
                mod_token: Default::default(),
                ident: item.ident,
                content: Some((Default::default(), items)),
                semi: Some(item.semi_token),
            });
        } else {
            unreachable!();
        }

        let mut new_items = crates;
        new_items.extend(file.items.drain(..));
        file.items = new_items;
    }
}

fn is_extern_crate(item: &syn::Item) -> bool {
    if let syn::Item::ExternCrate(_) = *item {
        true
    } else {
        false
    }
}

fn expand_mods(file: &mut syn::File, base_path: &Path) {
    for item in file.items.iter_mut() {
        if let syn::Item::Mod(ref mut item) = *item {
            if item.content.is_some() {
                continue;
            }
            let name = item.ident.to_string();
            let (base_path, code) = vec![
                (base_path.to_owned(), format!("{}.rs", name)),
                (base_path.join(&name), String::from("mod.rs")),
            ].into_iter()
                .flat_map(|(base_path, file_name)| {
                    read_file(&base_path.join(file_name)).map(|code| (base_path, code))
                })
                .next()
                .expect("mod not found");
            eprintln!("expanding mod {} in {}", name, base_path.to_str().unwrap());
            let mut file = syn::parse_file(&code).expect("failed to parse file");
            expand_mods(&mut file, &base_path);
            item.content = Some((Default::default(), file.items));
        }
    }
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
