use getopts::Options;
use quote::ToTokens;
use std::env;

mod bundle;
mod elimination;
mod mod_tree;
mod path;

fn main() {
    env_logger::init();

    let args = env::args();

    let mut opts = Options::new();
    opts.optflag("", "bin", "Bundle the binary crate");
    let root_file = if let Ok(m) = opts.parse(args) {
        if m.opt_present("bin") {
            "src/main.rs"
        } else {
            "src/lib.rs"
        }
    } else {
        let msg = opts.short_usage("bundler");
        eprintln!("{}", msg);
        return;
    };

    let res = bundle::bundle_file(root_file).expect("error");

    let tree = mod_tree::make_mod_tree(&res);
    // dbg!(tree);
    let mods = elimination::flatten_mods(tree);
    // dbg!(mods);

    println!("{}", res.to_token_stream().to_string());
}
