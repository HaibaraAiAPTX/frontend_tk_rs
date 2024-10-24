use std::{env::current_dir, sync::Arc};

use swc::{try_with_handler, Compiler};
use swc_common::{SourceMap, GLOBALS};

#[test]
fn test_run_js() {
    let cm = Arc::<SourceMap>::default();
    let c = Compiler::new(cm.clone());

    let mut path = current_dir().unwrap();
    path.push("tests\\test.ts");

    let output = GLOBALS
        .set(&Default::default(), || {
            try_with_handler(cm.clone(), Default::default(), |handler| {
                let fm = cm.load_file(&path).unwrap();
                c.process_js_file(fm.clone(), handler, &Default::default())
            })
        })
        .unwrap();

    assert!(!output.code.is_empty());
}
