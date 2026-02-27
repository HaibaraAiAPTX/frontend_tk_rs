use swagger_tk::model::OpenAPIObject;
use std::str::FromStr;

/// Mock OpenAPI spec for model parsing test
const MOCK_OPENAPI: &str = r###"
{
  "openapi": "3.1.0",
  "info": { "title": "mock-api", "version": "1.0.0" },
  "paths": {
    "/users": {
      "get": {
        "operationId": "getUsers",
        "responses": { "200": { "description": "Success" } }
      }
    }
  },
  "components": {
    "schemas": {
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

#[test]
fn model_test() {
    let open_api = OpenAPIObject::from_str(MOCK_OPENAPI);
    assert!(open_api.is_ok(), "{}", open_api.unwrap_err());
}
