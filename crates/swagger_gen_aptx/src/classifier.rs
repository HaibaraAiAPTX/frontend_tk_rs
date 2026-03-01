//! Custom query/mutation classification for @aptx code generation.
//!
//! This module provides `AptxQueryMutationPass` which extends the default classification
//! logic to handle cases where POST requests are used for data retrieval (e.g., list
//! queries with complex filters that exceed URL length limits).

use crate::META_SUPPORTS_QUERY;
use swagger_gen::pipeline::{EndpointItem, GeneratorInput, TransformPass};

/// Aptx-specific query/mutation classification pass.
///
/// ## Classification Rules
///
/// 1. **GET requests** → Query
/// 2. **POST requests with query-semantic names** → Query
///    - Names starting with `get` followed by uppercase letter (e.g., `getList`, `getTree`, `getPagedList`)
///    - Names starting with `query` (e.g., `queryUsers`, `queryByFilter`)
///    - Names starting with `search` (e.g., `searchItems`, `searchByKeyword`)
///    - Names starting with `fetch` (e.g., `fetchData`, `fetchRecords`)
///    - Names starting with `find` (e.g., `findByCondition`, `findResults`)
/// 3. **All other requests** → Mutation
///
/// ## Rationale
///
/// Some APIs use POST for list queries to avoid URL length limitations when passing
/// complex filter parameters. These endpoints are semantically queries and should
/// be treated as such in React Query (with caching, background refetching, etc.).
pub struct AptxQueryMutationPass;

impl TransformPass for AptxQueryMutationPass {
    fn name(&self) -> &'static str {
        "aptx-query-mutation"
    }

    fn apply(&self, input: &mut GeneratorInput) -> Result<(), String> {
        for endpoint in &mut input.endpoints {
            let is_query = classify_endpoint(endpoint);
            if is_query {
                endpoint.meta.insert(META_SUPPORTS_QUERY.to_string(), "true".to_string());
            }
        }
        Ok(())
    }
}

/// Determines if an endpoint should be classified as a query (vs mutation).
fn classify_endpoint(endpoint: &EndpointItem) -> bool {
    // GET is always a query
    if endpoint.method.eq_ignore_ascii_case("GET") {
        return true;
    }

    // For POST and other methods, check if the name suggests a query operation
    // Use operation_name (e.g., "getList") not export_name (e.g., "assignmentTypeGroupGetList")
    if endpoint.method.eq_ignore_ascii_case("POST")
        && is_query_semantic_name(&endpoint.operation_name)
    {
        return true;
    }

    false
}

/// Checks if the operation name suggests a query/read operation.
///
/// Matches common patterns like:
/// - `getXxx` (camelCase, e.g., `getList`, `getTree`, `getPagedList`)
/// - `queryXxx` (e.g., `queryUsers`, `queryByFilter`)
/// - `searchXxx` (e.g., `searchItems`, `searchByKeyword`)
/// - `fetchXxx` (e.g., `fetchData`, `fetchRecords`)
/// - `findXxx` (e.g., `findByCondition`, `findResults`)
/// - `listXxx` (e.g., `listUsers`, `listAll`)
fn is_query_semantic_name(name: &str) -> bool {
    // Query-semantic prefixes in camelCase format
    // The pattern is: lowercase prefix + uppercase letter + rest
    let query_prefixes = ["get", "query", "search", "fetch", "find", "list"];

    for prefix in query_prefixes {
        if let Some(rest) = name.strip_prefix(prefix) {
            // Ensure there's at least one character after the prefix and it's uppercase
            // This distinguishes `getList` (query) from `get` (too short) or `getting` (not camelCase)
            if !rest.is_empty() && rest.chars().next().is_some_and(|c| c.is_uppercase()) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    #[test]
    fn test_is_query_semantic_name() {
        // Should match - query patterns
        assert!(is_query_semantic_name("getList"));
        assert!(is_query_semantic_name("getTree"));
        assert!(is_query_semantic_name("getPagedList"));
        assert!(is_query_semantic_name("getUser"));
        assert!(is_query_semantic_name("getByCondition"));
        assert!(is_query_semantic_name("queryUsers"));
        assert!(is_query_semantic_name("queryByFilter"));
        assert!(is_query_semantic_name("searchItems"));
        assert!(is_query_semantic_name("searchByKeyword"));
        assert!(is_query_semantic_name("fetchData"));
        assert!(is_query_semantic_name("fetchRecords"));
        assert!(is_query_semantic_name("findByCondition"));
        assert!(is_query_semantic_name("findResults"));
        assert!(is_query_semantic_name("listUsers"));
        assert!(is_query_semantic_name("listAll"));

        // Should not match - mutation patterns
        assert!(!is_query_semantic_name("createUser"));
        assert!(!is_query_semantic_name("updateUser"));
        assert!(!is_query_semantic_name("deleteUser"));
        assert!(!is_query_semantic_name("add"));
        assert!(!is_query_semantic_name("remove"));
        assert!(!is_query_semantic_name("save"));
        assert!(!is_query_semantic_name("submit"));
        assert!(!is_query_semantic_name("get")); // Too short
        assert!(!is_query_semantic_name("getting")); // Not camelCase
        assert!(!is_query_semantic_name("")); // Empty
    }

    fn create_test_endpoint(method: &str, operation_name: &str) -> EndpointItem {
        // Simulate real-world scenario where export_name includes namespace prefix
        // e.g., operation_name = "getList" -> export_name = "assignmentTypeGroupGetList"
        let export_name = format!("test{}", to_pascal_case(operation_name));
        EndpointItem {
            namespace: vec!["test".to_string()],
            operation_name: operation_name.to_string(),
            export_name,
            builder_name: format!("buildTest{}Spec", to_pascal_case(operation_name)),
            summary: None,
            method: method.to_string(),
            path: "/test".to_string(),
            input_type_name: "void".to_string(),
            output_type_name: "void".to_string(),
            request_body_field: None,
            query_fields: vec![],
            path_fields: vec![],
            has_request_options: false,
            deprecated: false,
            meta: IndexMap::new(),
        }
    }

    fn to_pascal_case(s: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = true;
        for c in s.chars() {
            if c == '_' || c == '-' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push(c.to_uppercase().next().unwrap_or(c));
                capitalize_next = false;
            } else {
                result.push(c);
            }
        }
        result
    }

    #[test]
    fn test_classify_endpoint() {
        // GET should always be query
        let get_endpoint = create_test_endpoint("GET", "getUser");
        assert!(classify_endpoint(&get_endpoint));

        // POST with query name should be query
        let post_get_list = create_test_endpoint("POST", "getList");
        assert!(classify_endpoint(&post_get_list));

        let post_query = create_test_endpoint("POST", "queryUsers");
        assert!(classify_endpoint(&post_query));

        let post_search = create_test_endpoint("POST", "searchItems");
        assert!(classify_endpoint(&post_search));

        // POST with mutation name should be mutation
        let post_create = create_test_endpoint("POST", "createUser");
        assert!(!classify_endpoint(&post_create));

        let post_update = create_test_endpoint("POST", "updateUser");
        assert!(!classify_endpoint(&post_update));

        // PUT, DELETE should be mutation
        let put_endpoint = create_test_endpoint("PUT", "updateUser");
        assert!(!classify_endpoint(&put_endpoint));

        let delete_endpoint = create_test_endpoint("DELETE", "deleteUser");
        assert!(!classify_endpoint(&delete_endpoint));
    }

    #[test]
    fn test_classify_endpoint_with_namespace_prefix() {
        // Test that classification works correctly even when export_name has namespace prefix
        // This is the real-world case: operation_name="getList", export_name="assignmentTypeGroupGetList"
        let endpoint = create_test_endpoint("POST", "getList");
        // operation_name = "getList" -> should match "get" prefix
        // export_name = "TestGetList" -> would NOT match "get" prefix
        assert!(classify_endpoint(&endpoint));
    }

    #[test]
    fn test_aptx_query_mutation_pass() {
        let pass = AptxQueryMutationPass;

        let mut input = GeneratorInput {
            project: swagger_gen::pipeline::ProjectContext {
                package_name: "test".to_string(),
                api_base_path: None,
                terminals: vec![],
                retry_ownership: None,
            },
            endpoints: vec![
                create_test_endpoint("GET", "getUser"),
                create_test_endpoint("POST", "getList"),
                create_test_endpoint("POST", "createUser"),
                create_test_endpoint("PUT", "updateUser"),
                create_test_endpoint("DELETE", "deleteUser"),
            ],
            model_import: None,
            client_import: None,
            output_root: None,
        };

        pass.apply(&mut input).unwrap();

        // GET getUser -> query
        assert!(input.endpoints[0].meta.get(META_SUPPORTS_QUERY) == Some(&"true".to_string()));

        // POST getList -> query (special case)
        assert!(input.endpoints[1].meta.get(META_SUPPORTS_QUERY) == Some(&"true".to_string()));

        // POST createUser -> mutation (no supports_query meta)
        assert!(input.endpoints[2].meta.get(META_SUPPORTS_QUERY).is_none());

        // PUT updateUser -> mutation (no supports_query meta)
        assert!(input.endpoints[3].meta.get(META_SUPPORTS_QUERY).is_none());

        // DELETE deleteUser -> mutation (no supports_query meta)
        assert!(input.endpoints[4].meta.get(META_SUPPORTS_QUERY).is_none());
    }
}
