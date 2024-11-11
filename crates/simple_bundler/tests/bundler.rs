use std::env::current_dir;
use path_clean::PathClean;
use simple_bundler::SimpleBundler;

#[test]
fn bundler_test() {
    let root = current_dir().unwrap().join("tests");
    let bundler = SimpleBundler::new();
    let main_name = bundler.bundle(&root.join("./test.ts").clean());
    let output = root.join("dist");
    bundler.write(&output);
    assert!(main_name.contains("test"));
}
