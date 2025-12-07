use path_clean::PathClean;
use simple_bundler::Resolver;
use std::{env::current_dir, path::PathBuf};

#[test]
fn resolve_test() {
    let resolver = Resolver::default();
    let dir = current_dir().unwrap().join("../../test-bundler").clean();

    let index_resolution = resolver
        .resolve(&dir, &PathBuf::from("./src/index.ts"))
        .unwrap();
    assert_eq!(
        index_resolution.path(),
        PathBuf::from(&dir).join("./src/index.ts").clean()
    );

    let lodash_resolution = resolver.resolve(
        &index_resolution.path().to_path_buf(),
        &PathBuf::from("lodash-es"),
    );
    assert!(lodash_resolution.is_ok());

    let add_resolution = resolver.resolve(
        &lodash_resolution.unwrap().path().to_path_buf(),
        &PathBuf::from("./add.js"),
    );
    assert!(add_resolution.is_ok());

    let match_resolution = resolver.resolve(
        &add_resolution.unwrap().path().to_path_buf(),
        &PathBuf::from("./_createMathOperation.js"),
    );
    assert!(match_resolution.is_ok());

    dbg!(&match_resolution.unwrap().path());
}
