use swagger_tk::getter::get_schema_name_list;
use swagger_tk::model::OpenAPIObject;
use std::str::FromStr;

/// Mock OpenAPI spec with schemas for testing
const MOCK_OPENAPI: &str = r###"
{
  "openapi": "3.1.0",
  "info": { "title": "mock-api", "version": "1.0.0" },
  "paths": {},
  "components": {
    "schemas": {
      "Order": {
        "type": "object",
        "properties": {
          "id": { "type": "integer" },
          "petId": { "type": "integer" }
        }
      },
      "User": {
        "type": "object",
        "properties": {
          "id": { "type": "integer" },
          "name": { "type": "string" }
        }
      }
    }
  }
}
"###;

fn get_mock_openapi() -> OpenAPIObject {
    OpenAPIObject::from_str(MOCK_OPENAPI).expect("parse mock openapi fail")
}

#[test]
fn get_schema_name_list_test() {
    let open_api_object = get_mock_openapi();
    let schema_list = get_schema_name_list(&open_api_object);
    assert!(schema_list.is_some());
    let schemas = schema_list.unwrap();
    assert!(schemas.len() > 0);
    let order = "Order".to_string();
    let user = "User".to_string();
    assert!(schemas.contains(&&order));
    assert!(schemas.contains(&&user));
}
