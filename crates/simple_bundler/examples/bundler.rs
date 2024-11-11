use std::env::current_dir;

use path_clean::PathClean;
use simple_bundler::SimpleBundler;

fn main() {
    let root = current_dir().unwrap().join("test-bundler");
    let bundler = SimpleBundler::new();
    bundler.bundle(&root.join("./src/index.ts").clean());
    bundler.write(&root.join("dist"));
}
