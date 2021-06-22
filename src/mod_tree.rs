use std::mem;

use syn::visit::{self, Visit};

use crate::path::ModulePathBuf;

pub fn make_mod_tree(node: &syn::File) -> ModTree {
    let mut visitor = MakeModTree {
        current: Node::new(),
    };
    visitor.visit_file(node);
    ModTree::File(visitor.current)
}

#[derive(Clone, Debug)]
pub enum ModTree {
    File(Node),
    Mod(String, Node),
    Use(Vec<(String, ModulePathBuf)>),
    Generics(Node),
    Type(String),
    Block(Node),
}

#[derive(Clone, Debug)]
pub struct Node {
    pub items: Vec<ModTree>,
    pub accesses: Vec<ModulePathBuf>,
}

impl Node {
    fn new() -> Self {
        Self {
            items: vec![],
            accesses: vec![],
        }
    }
}

struct MakeModTree {
    current: Node,
}

impl<'ast> Visit<'ast> for MakeModTree {
    fn visit_path(&mut self, node: &'ast syn::Path) {
        self.current.accesses.push(parse_path(node));

        visit::visit_path(self, node);
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let mut temp = mem::replace(&mut self.current, Node::new());

        visit::visit_item_mod(self, node);

        mem::swap(&mut temp, &mut self.current);
        self.current
            .items
            .push(ModTree::Mod(node.ident.to_string(), temp));
    }

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        if node.leading_colon.is_some() {
            panic!("leading colon of path is unsupported");
        }
        self.current.items.push(ModTree::Use(parse_use(&node.tree)));

        visit::visit_item_use(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        let mut gen = Node::new();
        for param in &node.generics.params {
            if let syn::GenericParam::Type(ty) = param {
                gen.items.push(ModTree::Type(ty.ident.to_string()));
            }
        }
        let mut temp = mem::replace(&mut self.current, gen);

        visit::visit_item_impl(self, node);

        mem::swap(&mut temp, &mut self.current);
        self.current.items.push(ModTree::Generics(temp));
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let mut gen = Node::new();
        for param in &node.sig.generics.params {
            if let syn::GenericParam::Type(ty) = param {
                gen.items.push(ModTree::Type(ty.ident.to_string()));
            }
        }
        let mut temp = mem::replace(&mut self.current, gen);

        visit::visit_item_fn(self, node);

        mem::swap(&mut temp, &mut self.current);
        self.current.items.push(ModTree::Generics(temp));
    }

    fn visit_block(&mut self, node: &'ast syn::Block) {
        let mut temp = mem::replace(&mut self.current, Node::new());

        visit::visit_block(self, node);

        mem::swap(&mut temp, &mut self.current);
        self.current.items.push(ModTree::Block(temp));
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.current
            .items
            .push(ModTree::Type(node.ident.to_string()));

        visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        self.current
            .items
            .push(ModTree::Type(node.ident.to_string()));

        visit::visit_item_enum(self, node);
    }

    fn visit_item_union(&mut self, node: &'ast syn::ItemUnion) {
        self.current
            .items
            .push(ModTree::Type(node.ident.to_string()));

        visit::visit_item_union(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        self.current
            .items
            .push(ModTree::Type(node.ident.to_string()));

        visit::visit_item_trait(self, node);
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        self.current
            .items
            .push(ModTree::Type(node.ident.to_string()));

        visit::visit_item_type(self, node);
    }
}

fn parse_path(node: &syn::Path) -> ModulePathBuf {
    if node.leading_colon.is_some() {
        panic!("leading colon of path is unsupported");
    }
    node.segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect()
}

fn parse_use(node: &syn::UseTree) -> Vec<(String, ModulePathBuf)> {
    match node {
        syn::UseTree::Name(name) => vec![(name.ident.to_string(), name.ident.to_string().into())],
        syn::UseTree::Rename(rename) => {
            vec![(rename.rename.to_string(), rename.ident.to_string().into())]
        }
        syn::UseTree::Glob(_glob) => vec![("*".to_string(), "*".into())],
        syn::UseTree::Path(path) => {
            let name = path.ident.to_string();
            let mut res = parse_use(&path.tree);
            for (rename, path) in &mut res {
                if rename == "self" {
                    *rename = name.clone();
                    *path = name.clone().into();
                } else {
                    *path = ModulePathBuf::from(name.clone()).join(&*path);
                }
            }
            res
        }
        syn::UseTree::Group(group) => group
            .items
            .iter()
            .flat_map(|item| parse_use(item))
            .collect(),
    }
}
