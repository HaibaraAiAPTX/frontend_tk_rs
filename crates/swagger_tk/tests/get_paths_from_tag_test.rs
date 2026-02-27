use swagger_tk::getter::get_paths_from_tag;
use swagger_tk::model::OpenAPIObject;
use std::str::FromStr;

/// Mock OpenAPI spec with pet tag for testing
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
    "/pet/{id}": {
      "get": {
        "tags": ["pet"],
        "operationId": "getPetById",
        "summary": "Get pet by ID",
        "parameters": [
          { "name": "id", "in": "path", "required": true, "schema": { "type": "integer" } }
        ],
        "responses": { "200": { "description": "Success" } }
      }
    },
    "/store/order": {
      "get": {
        "tags": ["store"],
        "operationId": "getOrders",
        "summary": "Get orders",
        "responses": { "200": { "description": "Success" } }
      }
    }
  },
  "components": {
    "schemas": {
      "Order": {
        "type": "object",
        "properties": {
          "id": { "type": "integer" },
          "petId": { "type": "integer" },
          "quantity": { "type": "integer" }
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
fn get_paths_from_tag_test() {
    let open_api_object = get_mock_openapi();
    let paths = get_paths_from_tag(&open_api_object, "pet");
    assert!(paths.len() > 0, "没有找到 pet 标签的路径");
}
