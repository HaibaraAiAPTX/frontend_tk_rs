use futures::executor::block_on;
use quickjs_runtime::{
    builder::QuickJsRuntimeBuilder,
    jsutils::Script,
    values::{JsValueConvertable, JsValueFacade},
};

#[test]
fn console_test() {
    let rt = QuickJsRuntimeBuilder::new().build();
    let script = r#"
        console.log('Hello, %s!', 'world');
        console.error('This is an error message.');
        console.warn('This is a warning message.');
        console.info('This is an info message.');
        console.debug('This is a debug message.');
    "#;
    rt.eval_sync(None, Script::new("console.es", script))
        .expect("script failed");
    block_on(rt.eval(None, Script::new("test.js", script))).expect("script fail");
}

#[test]
fn run_main_test() {
    let script = r#"
        globalThis.result = 42;
    "#;
    let rt = QuickJsRuntimeBuilder::new().build();
    let res = rt.eval_sync(None, Script::new("test.js", script)).ok();
    if let Some(js_value) = res {
        let ret = js_value.get_i32();
        assert_eq!(ret, 42);
    } else {
        panic!("script failed");
    }
}

#[tokio::test]
async fn top_return_promise_test() {
    let script = r#"
        async function fetchData() {
            return new Promise((resolve) => {
                setTimeout(() => {
                    resolve(100);
                }, 100);
            });
        }
        globalThis.result = fetchData();
    "#;
    let rt = QuickJsRuntimeBuilder::new().build();
    let res = rt.eval_sync(None, Script::new("test.js", script)).ok();
    if let Some(JsValueFacade::JsPromise { cached_promise }) = res {
        let res = cached_promise
            .get_promise_result()
            .await
            .expect("promise then fail");
        match res {
            Ok(v) => {
                if v.is_i32() {
                    let ret = v.get_i32();
                    assert_eq!(ret, 100);
                } else {
                    panic!("unexpected return type");
                }
            }
            Err(_) => panic!("promise rejected"),
        }
    } else {
        panic!("script failed");
    }
}

#[test]
fn invoke_fn_test() {
    let script = r#"
        function add(a, b) {
            return a + b;
        }
    "#;
    let rt = QuickJsRuntimeBuilder::new().build();
    rt.eval_sync(None, Script::new("test.js", script)).ok();
    let res = rt.invoke_function_sync(
        None,
        &[],
        "add",
        vec![1.to_js_value_facade(), 2.to_js_value_facade()],
    );
    if let Ok(js_value) = res {
        let sum = js_value.get_i32();
        assert_eq!(sum, 3);
    } else {
        panic!("invoke function failed");
    }
}

#[tokio::test]
async fn invoke_promise_fn_test() {
    let script = r#"
        function addAsync(a, b) {
            return new Promise((resolve) => {
                setTimeout(() => {
                    resolve(a + b);
                }, 100);
            });
        }
    "#;
    let rt = QuickJsRuntimeBuilder::new().build();
    rt.eval_sync(None, Script::new("test.js", script)).ok();
    let res = rt
        .invoke_function(
            None,
            &[],
            "addAsync",
            vec![3.to_js_value_facade(), 4.to_js_value_facade()],
        )
        .await;
    if let Ok(JsValueFacade::JsPromise { cached_promise }) = res {
        let res = cached_promise
            .get_promise_result()
            .await
            .expect("promise fail");
        match res {
            Ok(v) => {
                if v.is_i32() {
                    let sum = v.get_i32();
                    assert_eq!(sum, 7);
                } else {
                    panic!("unexpected return type");
                }
            }
            Err(_) => panic!("promise rejected"),
        }
    }
}
