use std::str::FromStr;
use swagger_gen::pipeline::{
    build_dry_run_plan, build_ir_snapshot_json, build_report_json, parse_openapi_to_ir,
};
use swagger_tk::model::OpenAPIObject;

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

/// Comprehensive OpenAPI spec covering all common HTTP parameter scenarios
const PARAM_SCENARIOS_OPENAPI: &str = r###"
{
  "openapi": "3.1.0",
  "info": { "title": "param-scenarios-api", "version": "1.0.0" },
  "paths": {
    "/items": {
      "get": {
        "operationId": "listItems",
        "summary": "No params at all",
        "responses": { "200": { "description": "OK" } }
      }
    },
    "/items/create": {
      "post": {
        "operationId": "createItem",
        "summary": "Body only",
        "requestBody": {
          "content": {
            "application/json": {
              "schema": { "$ref": "#/components/schemas/Item" }
            }
          }
        },
        "responses": { "201": { "description": "Created" } }
      }
    },
    "/items/{id}": {
      "get": {
        "operationId": "getItemById",
        "summary": "Path param only",
        "parameters": [
          { "name": "id", "in": "path", "required": true, "schema": { "type": "string" } }
        ],
        "responses": { "200": { "description": "OK" } }
      }
    },
    "/items/search": {
      "get": {
        "operationId": "searchItems",
        "summary": "Single query param",
        "parameters": [
          { "name": "keyword", "in": "query", "required": false, "schema": { "type": "string" } }
        ],
        "responses": { "200": { "description": "OK" } }
      }
    },
    "/items/filter": {
      "get": {
        "operationId": "filterItems",
        "summary": "Multiple query params",
        "parameters": [
          { "name": "page", "in": "query", "required": false, "schema": { "type": "integer" } },
          { "name": "size", "in": "query", "required": false, "schema": { "type": "integer" } }
        ],
        "responses": { "200": { "description": "OK" } }
      }
    },
    "/items/{id}/detail": {
      "get": {
        "operationId": "getItemDetail",
        "summary": "Path + query params",
        "parameters": [
          { "name": "id", "in": "path", "required": true, "schema": { "type": "string" } },
          { "name": "expand", "in": "query", "required": false, "schema": { "type": "boolean" } }
        ],
        "responses": { "200": { "description": "OK" } }
      }
    },
    "/items/{id}/update": {
      "put": {
        "operationId": "updateItem",
        "summary": "Path param + body",
        "parameters": [
          { "name": "id", "in": "path", "required": true, "schema": { "type": "string" } }
        ],
        "requestBody": {
          "content": {
            "application/json": {
              "schema": { "$ref": "#/components/schemas/Item" }
            }
          }
        },
        "responses": { "200": { "description": "OK" } }
      }
    },
    "/items/batch": {
      "post": {
        "operationId": "batchCreate",
        "summary": "Query param + body",
        "parameters": [
          { "name": "notify", "in": "query", "required": false, "schema": { "type": "boolean" } }
        ],
        "requestBody": {
          "content": {
            "application/json": {
              "schema": { "$ref": "#/components/schemas/Item" }
            }
          }
        },
        "responses": { "201": { "description": "Created" } }
      }
    },
    "/items/delete": {
      "delete": {
        "operationId": "deleteItem",
        "summary": "DELETE with query param",
        "parameters": [
          { "name": "id", "in": "query", "required": true, "schema": { "type": "string" } }
        ],
        "responses": { "200": { "description": "OK" } }
      }
    }
  },
  "components": {
    "schemas": {
      "Item": {
        "type": "object",
        "properties": {
          "id": { "type": "string" },
          "name": { "type": "string" }
        },
        "required": ["id", "name"]
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

/// Find an endpoint by (method, path) in the parsed IR
fn find_endpoint<'a>(
    ir: &'a swagger_gen::pipeline::GeneratorInput,
    method: &str,
    path: &str,
) -> Option<&'a swagger_gen::pipeline::EndpointItem> {
    ir.endpoints
        .iter()
        .find(|e| e.method == method && e.path == path)
}

#[test]
fn input_type_name_all_param_scenarios() {
    let open_api: OpenAPIObject =
        OpenAPIObject::from_str(PARAM_SCENARIOS_OPENAPI).expect("parse param scenarios fail");
    let ir = parse_openapi_to_ir(&open_api).expect("parse openapi to ir fail");

    // 1. No params → input_type_name = "void"
    let ep = find_endpoint(&ir, "GET", "/items").expect("GET /items");
    assert_eq!(ep.input_type_name, "void");
    assert!(ep.query_fields.is_empty());
    assert!(ep.path_fields.is_empty());

    // 2. Body only → input_type_name = body type, not inline
    let ep = find_endpoint(&ir, "POST", "/items/create").expect("POST /items/create");
    assert_eq!(ep.input_type_name, "Item");
    assert!(ep.query_fields.is_empty());
    assert!(ep.path_fields.is_empty());

    // 3. Path param only → input_type_name = "void", path_fields = ["id"]
    let ep = find_endpoint(&ir, "GET", "/items/{id}").expect("GET /items/{id}");
    assert_eq!(ep.input_type_name, "void");
    assert!(ep.query_fields.is_empty());
    assert_eq!(ep.path_fields, vec!["id"]);

    // 4. Single query param → input_type_name = "void", query_fields = ["keyword"]
    let ep = find_endpoint(&ir, "GET", "/items/search").expect("GET /items/search");
    assert_eq!(ep.input_type_name, "void");
    assert_eq!(ep.query_fields, vec!["keyword"]);
    assert_eq!(ep.query_params.len(), 1);
    assert_eq!(ep.query_params[0].name, "keyword");
    assert_eq!(ep.query_params[0].type_name, "string");
    assert!(!ep.query_params[0].required);
    assert!(ep.path_fields.is_empty());

    // 5. Multiple query params → input_type_name = "void", query_fields = ["page", "size"]
    let ep = find_endpoint(&ir, "GET", "/items/filter").expect("GET /items/filter");
    assert_eq!(ep.input_type_name, "void");
    assert_eq!(ep.query_fields, vec!["page", "size"]);
    assert!(ep.path_fields.is_empty());

    // 6. Path + query params → input_type_name = "void"
    let ep = find_endpoint(&ir, "GET", "/items/{id}/detail").expect("GET /items/{id}/detail");
    assert_eq!(ep.input_type_name, "void");
    assert_eq!(ep.path_fields, vec!["id"]);
    assert_eq!(ep.query_fields, vec!["expand"]);
    assert_eq!(ep.path_params.len(), 1);
    assert_eq!(ep.path_params[0].name, "id");
    assert_eq!(ep.path_params[0].type_name, "string");
    assert!(ep.path_params[0].required);
    assert_eq!(ep.query_params.len(), 1);
    assert_eq!(ep.query_params[0].name, "expand");
    assert_eq!(ep.query_params[0].type_name, "boolean");
    assert!(!ep.query_params[0].required);

    // 7. Path param + body → input_type_name = body type (not inline merged type)
    let ep = find_endpoint(&ir, "PUT", "/items/{id}/update").expect("PUT /items/{id}/update");
    assert_eq!(ep.input_type_name, "Item");
    assert_eq!(ep.path_fields, vec!["id"]);
    assert!(ep.query_fields.is_empty());

    // 8. Query param + body → input_type_name = body type (not inline merged type)
    let ep = find_endpoint(&ir, "POST", "/items/batch").expect("POST /items/batch");
    assert_eq!(ep.input_type_name, "Item");
    assert_eq!(ep.query_fields, vec!["notify"]);
    assert!(ep.path_fields.is_empty());

    // 9. DELETE with query param → input_type_name = "void"
    let ep = find_endpoint(&ir, "DELETE", "/items/delete").expect("DELETE /items/delete");
    assert_eq!(ep.input_type_name, "void");
    assert_eq!(ep.query_fields, vec!["id"]);
    assert_eq!(ep.query_params.len(), 1);
    assert_eq!(ep.query_params[0].name, "id");
    assert_eq!(ep.query_params[0].type_name, "string");
    assert!(ep.query_params[0].required);
    assert!(ep.path_fields.is_empty());
}
