use std::env::current_dir;
use deno_core::{serde_json, v8};
use path_clean::PathClean;
use ts_runtime::{run_func, run_main_file};

#[tokio::test]
async fn run_test_bundler_test() {
    let entry_path = current_dir().unwrap().join("../../test-bundler/src/index.ts").clean();
    let result = run_func(&entry_path, "getSum", |_| vec![]).await.unwrap();
    assert_eq!(result, serde_json::json!(6));
    let result = run_func(&entry_path, "asyncGetSum", |_| vec![]).await.unwrap();
    assert_eq!(result, serde_json::json!(6));
}

#[tokio::test]
async fn runner1_test() {
    let entry_path = current_dir().unwrap().join("./tests/runner1.ts").clean();
    let result = run_main_file(&entry_path).await.unwrap();
    assert_eq!(result, serde_json::json!(10));
}

#[tokio::test]
async fn runner2_test() {
    let entry_path = current_dir().unwrap().join("./tests/runner2.ts").clean();
    let result = run_main_file(&entry_path).await.unwrap();
    assert_eq!(result, serde_json::json!("hello world async result"));
}

#[tokio::test]
async fn runner3_test() {
    let entry_path = current_dir().unwrap().join("./tests/runner3.ts").clean();
    let result = run_func(&entry_path, "main", |_| vec![]).await.unwrap();
    assert_eq!(result, serde_json::json!("hello world"));
}

#[tokio::test]
async fn runner4_test() {
    let entry_path = current_dir().unwrap().join("./tests/runner4.ts").clean();
    let result = run_func(&entry_path, "double_count", |scope| {
        let count = v8::Number::new(scope, 2.0).cast();
        let count = v8::Global::new(scope, count);
        vec![count]
    }).await.unwrap();
    assert_eq!(result, serde_json::json!(4));
    let result = run_func(&entry_path, "async_double_count", |scope| {
        let count = v8::Number::new(scope, 2.0).cast();
        let count = v8::Global::new(scope, count);
        vec![count]
    }).await.unwrap();
    assert_eq!(result, serde_json::json!(4));
}
