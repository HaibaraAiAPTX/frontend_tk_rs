use swagger_tk::getter::{get_tags, get_tags_from_open_api};
use swagger_tk::model::OpenAPIObject;
use std::str::FromStr;

/// Mock OpenAPI spec with tags for testing
const MOCK_OPENAPI: &str = r###"
{
  "openapi": "3.1.0",
  "info": { "title": "mock-api", "version": "1.0.0" },
  "tags": [
    { "name": "pet", "description": "Pet operations" },
    { "name": "store", "description": "Store operations" }
  ],
  "paths": {
    "/pet": {
      "get": {
        "tags": ["pet"],
        "operationId": "getPets",
        "responses": { "200": { "description": "Success" } }
      }
    },
    "/store/order": {
      "get": {
        "tags": ["store"],
        "operationId": "getOrders",
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
fn get_tags_test() {
    let open_api_object = get_mock_openapi();
    let tags = get_tags(&open_api_object);
    assert!(tags.len() > 0);
}

#[test]
fn get_tags_from_open_api_test() {
    let open_api_object = get_mock_openapi();
    let tags = get_tags_from_open_api(&open_api_object);
    assert!(tags.is_some());
}
