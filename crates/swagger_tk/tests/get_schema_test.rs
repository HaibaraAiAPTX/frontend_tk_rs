use swagger_tk::getter::get_schema_by_name;
use swagger_tk::model::OpenAPIObject;
use std::str::FromStr;

/// Mock OpenAPI spec with Order schema for testing
const MOCK_OPENAPI: &str = r###"
{
  "openapi": "3.1.0",
  "info": { "title": "mock-api", "version": "1.0.0" },
  "paths": {},
  "components": {
    "schemas": {
      "Order": {
        "type": "object",
        "description": "Order information",
        "properties": {
          "id": { "type": "integer" },
          "petId": { "type": "integer" },
          "quantity": { "type": "integer" }
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
fn get_schema_test() {
    let open_api_object = get_mock_openapi();
    let schema = get_schema_by_name(&open_api_object, "Order");
    assert!(schema.is_some());
}
