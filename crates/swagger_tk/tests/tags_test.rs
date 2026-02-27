use swagger_tk::getter::{get_tags_from_path_item, get_tags_from_paths};
use swagger_tk::model::OpenAPIObject;
use std::str::FromStr;

/// Mock OpenAPI spec with pet path for testing
const MOCK_OPENAPI: &str = r###"
{
  "openapi": "3.1.0",
  "info": { "title": "mock-api", "version": "1.0.0" },
  "paths": {
    "/pet": {
      "get": {
        "tags": ["pet"],
        "operationId": "getPets",
        "summary": "Get all pets",
        "responses": { "200": { "description": "Success" } }
      },
      "post": {
        "tags": ["pet"],
        "operationId": "createPet",
        "summary": "Create a pet",
        "responses": { "201": { "description": "Created" } }
      }
    },
    "/store": {
      "get": {
        "tags": ["store"],
        "operationId": "getStore",
        "responses": { "200": { "description": "Success" } }
      }
    }
  },
  "components": { "schemas": {} }
}
"###;

fn get_mock_openapi() -> OpenAPIObject {
    OpenAPIObject::from_str(MOCK_OPENAPI).expect("parse mock openapi fail")
}

#[test]
fn tags_test() {
    let open_api_object = get_mock_openapi();
    let paths = &open_api_object.paths;
    assert!(paths.is_some(), "paths 不能为空");
    let paths = paths.as_ref().unwrap();
    let path_item = paths.get("/pet");
    assert!(path_item.is_some(), "没有找到 /pet 路径");
    let tags = get_tags_from_path_item(&path_item.unwrap());
    assert!(tags.len() > 0);
    let tags = get_tags_from_paths(paths);
    assert!(tags.len() > 0);
}
