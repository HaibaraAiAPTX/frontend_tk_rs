use path_clean::PathClean;
use std::env::current_dir;
use ts_runtime::{run_func, run_main_file};

#[tokio::test]
async fn test_us1_run_main_file_async() {
    let entry_path = current_dir().unwrap().join("tests/utils.ts").clean();
    let result = run_main_file(&entry_path).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_us1_run_main_file() {
    let entry_path = current_dir().unwrap().join("tests/runner1.ts").clean();
    let result = run_main_file(&entry_path).await;
    assert!(result.is_ok());
    dbg!(result.unwrap().to_serde_value().await.unwrap());
}

#[tokio::test]
async fn test_us1_run_func_async() {
    let entry_path = current_dir().unwrap().join("tests/utils.ts").clean();
    let result = run_func(&entry_path, "asyncHello".to_string(), |_| vec![]).await;
    if let Err(e) = &result {
        println!("Error: {}", e);
    }
    assert!(result.is_ok());
    let val = result.unwrap();
    assert_eq!(val.get_str(), "hello async");
}

#[tokio::test]
async fn test_us2_set_timeout() {
    let entry_path = current_dir().unwrap().join("tests/utils.ts").clean();
    let result = run_func(&entry_path, "sleep".to_string(), |realm| {
        vec![realm.create_i32(10).unwrap()]
    })
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_us2_set_interval() {
    let entry_path = current_dir().unwrap().join("tests/utils.ts").clean();
    let result = run_func(&entry_path, "intervalCount".to_string(), |realm| {
        vec![realm.create_i32(10).unwrap(), realm.create_i32(3).unwrap()]
    })
    .await;
    assert!(result.is_ok());
    let val = result.unwrap();
    assert_eq!(val.get_i32(), 3);
}
