use std::env::current_dir;

use oxc_resolver::{ResolveOptions, Resolver, TsconfigOptions};

#[test]
fn resolver_test() {
    let entry = current_dir().unwrap().join("tests");

    let resolver = Resolver::new(ResolveOptions {
        extensions: vec![".js".into(), ".ts".into()],
        condition_names: vec!["import".into(), "node".into()],
        extension_alias: vec![(".js".into(), vec![".ts".into(), ".js".into()])],
        main_fields: vec!["module".into(), "main".into()],
        tsconfig: Some(TsconfigOptions { config_file: todo!(), references: todo!() }),
        ..ResolveOptions::default()
    });

    let files = resolver.resolve(entry, "./test.ts").unwrap();
    println!("{:?}", files);
}
