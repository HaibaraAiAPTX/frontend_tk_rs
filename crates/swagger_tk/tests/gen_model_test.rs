use swagger_tk::gen::TypescriptDeclarationGen;
use utils::get_open_api_object;

mod utils;

#[test]
fn gen_declaration_test() {
    let open_api_object = get_open_api_object(Some("打卡.json"));
    let model_gen = TypescriptDeclarationGen {
        open_api: &open_api_object,
    };

    let model_text = model_gen.gen_declaration_by_name("AddAfterClassAssignmentRequestModel");
    assert!(model_text.is_ok());
    let model_list = model_gen.gen_declarations();
    assert!(model_list.is_ok());
    for (_, text) in model_list.unwrap() {
        println!("{text}");
    }
}
