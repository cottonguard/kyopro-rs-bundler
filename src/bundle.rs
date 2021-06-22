use std::{
    fs,
    io::Read,
    mem,
    path::{Path, PathBuf},
};

use anyhow::Result;
use log::info;
use syn::fold::{self, Fold};

use crate::path::{ModulePath, ModulePathBuf};

pub fn bundle_file(src: impl AsRef<Path>) -> Result<syn::File> {
    let src = src.as_ref();
    let tree = parse_rs_file(src)?;
    let mut bundler = Bundler::new(src);
    Ok(bundler.fold_file(tree))
}

pub fn parse_rs_file(src: impl AsRef<Path>) -> Result<syn::File> {
    let src = src.as_ref();
    info!("trying to open: {:?}", src);
    let mut file = fs::File::open(src)?;
    info!("opened: {:?}", src);
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    let tree = syn::parse_file(&buf)?;
    Ok(tree)
}

struct Bundler {
    src_dir: PathBuf,
    current_file: PathBuf,
    mod_path: ModulePathBuf,
}

impl Bundler {
    fn new(root_file: impl AsRef<Path>) -> Self {
        let root_file = root_file.as_ref();

        Self {
            current_file: root_file.into(),
            src_dir: root_file.parent().unwrap().into(),
            mod_path: ModulePathBuf::from("crate"),
        }
    }
}

impl Fold for Bundler {
    fn fold_item_mod(&mut self, mut node: syn::ItemMod) -> syn::ItemMod {
        let name = node.ident.to_string();

        self.mod_path.push(&name);

        if node.content.is_none() {
            let mut path_attr = None;

            if let Some(attr) = node.attrs.iter().find(|attr| {
                attr.path
                    .get_ident()
                    .map(|ident| ident == "path")
                    .unwrap_or(false)
            }) {
                let meta = attr.parse_meta().expect("parse failed");
                if let syn::Meta::NameValue(name_value) = meta {
                    if let syn::Lit::Str(path) = name_value.lit {
                        path_attr = Some(PathBuf::from(path.value()));
                    }
                }
            }

            let file_path = if let Some(path_attr) = path_attr {
                let current_dir = self.current_file.parent().unwrap();
                current_dir.join(path_attr)
            } else {
                let mut file_path = to_file_path(&self.src_dir, &self.mod_path);
                file_path.set_extension("rs");
                if !file_path.is_file() {
                    file_path.set_extension("");
                    file_path.push("mod.rs");
                }
                file_path
            };

            let tree = parse_rs_file(&file_path);
            let tree = tree.unwrap();

            let current_file = mem::replace(&mut self.current_file, file_path);

            let tree = self.fold_file(tree);

            self.current_file = current_file;

            node.content = Some((syn::token::Brace::default(), tree.items));
        }

        let res = fold::fold_item_mod(self, node);

        self.mod_path.pop();

        res
    }
}

fn to_file_path(src_dir: &Path, path: impl AsRef<ModulePath>) -> PathBuf {
    let path = path.as_ref();
    debug_assert!(path.is_crate_path());

    let mut res = src_dir.to_path_buf();

    for seg in path.as_str().split("::").skip(1) {
        res.push(seg);
    }

    res
}
