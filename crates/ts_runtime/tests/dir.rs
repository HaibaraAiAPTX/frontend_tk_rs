use path_clean::PathClean;
use std::env::current_dir;
use ts_runtime::{compiler, get_js_output_cache_dir};

#[test]
fn cache_test() {
    let cache_dir = get_js_output_cache_dir();
    assert!(cache_dir.is_ok());
    println!("{:?}", cache_dir.unwrap().display());
}

#[test]
fn compile_test() {
    let entry_path = current_dir()
        .unwrap()
        .join("../../test-bundler/src/index.ts")
        .clean();
    let entry_file_path = compiler(&entry_path).unwrap();
    assert!(entry_file_path.exists());
    println!("{:?}", entry_file_path.display());
}
