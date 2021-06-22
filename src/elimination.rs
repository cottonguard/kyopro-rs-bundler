use std::collections::{HashMap, HashSet, VecDeque, hash_map::Entry};

use crate::{
    mod_tree::{ModTree, Node},
    path::{ModulePath, ModulePathBuf},
};

pub fn flatten_mods(tree: ModTree) -> HashMap<ModulePathBuf, Node> {
    let mut res = HashMap::new();
    flatten_mods_rec(&mut res, ModulePath::new(""), tree);
    res
}

fn flatten_mods_rec(res: &mut HashMap<ModulePathBuf, Node>, path: &ModulePath, tree: ModTree) {
    let mut inner = |path: ModulePathBuf, mut node: Node| {
        // node.items.drain_filter()
        let mut i = 0;
        while i < node.items.len() {
            if matches!(&node.items[i], ModTree::Mod(_, _)) {
                let child_mod = node.items.remove(i);
                flatten_mods_rec(res, &*path, child_mod);
            } else {
                i += 1;
            }
        }

        match res.entry(path) {
            Entry::Vacant(e) => {
                e.insert(node);
            }
            Entry::Occupied(mut e) => {
                let x = e.get_mut();
                x.items.extend(node.items);
                x.accesses.extend(node.accesses);
            }
        }
    };

    match tree {
        ModTree::File(node) => {
            let path = ModulePathBuf::from("crate");
            inner(path, node);
        }
        ModTree::Mod(name, node) => {
            let path = path.join(name);
            inner(path, node);
        }
        _ => panic!("the tree is not File or Mod"),
    }
}

pub fn collect_reachablity(mods: &HashMap<ModulePathBuf, Node>) {
    let mut que: VecDeque<_> = Some(ModulePathBuf::from("crate")).into_iter().collect();
    let mut reached: HashSet<_> = Some(ModulePathBuf::from("crate")).into_iter().collect();

    while let Some(name) = que.pop_front() {
        let node = mods.get(&name).expect("mod is not found");

        for item in &node.items {
            match item {
                ModTree::Block(inner) | ModTree::Generics(inner) => {}
                _ => {}
            }
        }
    }
}

fn proc_mod(node: Node, scopes: &mut Vec<Scope>) {
    let mut aliases = HashMap::new();
    for item in node.items {
        match item {
            ModTree::Use(list) => {
                for (name, path) in list {
                    aliases.insert(name, path);
                }
            }
            _ => {}
        }
    }

    // let mut resolved = HashMap::new();
    // let mut stack = vec![];
}

struct Scope {
    types: HashSet<String>,
    aliases: HashMap<String, ModulePathBuf>,
}
