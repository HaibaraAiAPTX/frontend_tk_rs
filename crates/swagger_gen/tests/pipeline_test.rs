use swagger_gen::pipeline::{
    build_dry_run_plan, build_ir_snapshot_json, build_report_json, parse_openapi_to_ir,
};
use swagger_tk::model::OpenAPIObject;
use std::str::FromStr;

/// Mock OpenAPI spec with endpoints for pipeline testing
const MOCK_OPENAPI: &str = r###"
{
  "openapi": "3.1.0",
  "info": { "title": "mock-api", "version": "1.0.0" },
  "paths": {
    "/users": {
      "get": {
        "operationId": "getUsers",
        "summary": "Get all users",
        "responses": {
          "200": {
            "description": "Success",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": { "$ref": "#/components/schemas/User" }
                }
              }
            }
          }
        }
      },
      "post": {
        "operationId": "createUser",
        "summary": "Create a user",
        "requestBody": {
          "content": {
            "application/json": {
              "schema": { "$ref": "#/components/schemas/User" }
            }
          }
        },
        "responses": {
          "201": { "description": "Created" }
        }
      }
    },
    "/users/{id}": {
      "get": {
        "operationId": "getUserById",
        "summary": "Get user by ID",
        "parameters": [
          {
            "name": "id",
            "in": "path",
            "required": true,
            "schema": { "type": "integer" }
          }
        ],
        "responses": {
          "200": {
            "description": "Success",
            "content": {
              "application/json": {
                "schema": { "$ref": "#/components/schemas/User" }
              }
            }
          }
        }
      }
    },
    "/orders": {
      "get": {
        "operationId": "getOrders",
        "summary": "Get all orders",
        "responses": {
          "200": {
            "description": "Success",
            "content": {
              "application/json": {
                "schema": {
                  "type": "array",
                  "items": { "$ref": "#/components/schemas/Order" }
                }
              }
            }
          }
        }
      }
    }
  },
  "components": {
    "schemas": {
      "User": {
        "type": "object",
        "properties": {
          "id": { "type": "integer" },
          "name": { "type": "string" },
          "email": { "type": "string" }
        },
        "required": ["id", "name"]
      },
      "Order": {
        "type": "object",
        "properties": {
          "id": { "type": "integer" },
          "userId": { "type": "integer" },
          "amount": { "type": "number" }
        },
        "required": ["id", "userId"]
      }
    }
  }
}
"###;

fn get_mock_openapi() -> OpenAPIObject {
    OpenAPIObject::from_str(MOCK_OPENAPI).expect("parse mock openapi fail")
}

#[test]
fn parse_openapi_to_ir_test() {
    let open_api_object = get_mock_openapi();
    let ir = parse_openapi_to_ir(&open_api_object).expect("parse openapi to ir fail");
    assert!(!ir.endpoints.is_empty());
}

#[test]
fn build_dry_run_plan_test() {
    let open_api_object = get_mock_openapi();
    let plan = build_dry_run_plan(&open_api_object).expect("build dry run plan fail");
    assert!(plan.endpoint_count > 0);
    assert!(!plan.transform_steps.is_empty());
    assert!(plan.metrics.total_ms >= plan.metrics.parse_ms);
}

#[test]
fn build_ir_snapshot_json_test() {
    let open_api_object = get_mock_openapi();
    let snapshot = build_ir_snapshot_json(&open_api_object).expect("build ir snapshot fail");
    assert!(snapshot.contains("endpoints"));
}

#[test]
fn build_report_json_test() {
    let open_api_object = get_mock_openapi();
    let report = build_report_json(&open_api_object).expect("build report json fail");
    assert!(report.contains("renderer_reports"));
    assert!(report.contains("metrics"));
}
