use super::model::GeneratorInput;

pub trait TransformPass {
    fn name(&self) -> &'static str;
    fn apply(&self, input: &mut GeneratorInput) -> Result<(), String>;
}

/// Normalizes endpoint data: sorting, namespace defaults, and validation.
/// This pass does NOT set query/mutation classification - use DefaultQueryMutationPass or a custom pass for that.
pub struct NormalizeEndpointPass;

impl TransformPass for NormalizeEndpointPass {
    fn name(&self) -> &'static str {
        "normalize-endpoint"
    }

    fn apply(&self, input: &mut GeneratorInput) -> Result<(), String> {
        input.endpoints.sort_by(|a, b| {
            (&a.path, &a.method, &a.operation_name).cmp(&(&b.path, &b.method, &b.operation_name))
        });

        for endpoint in &mut input.endpoints {
            if endpoint.namespace.is_empty() {
                endpoint.namespace.push("default".to_string());
            }
            if endpoint.operation_name.trim().is_empty() {
                return Err(format!(
                    "operation_name is empty for endpoint {} {}",
                    endpoint.method, endpoint.path
                ));
            }
            if endpoint.export_name.trim().is_empty() {
                return Err(format!(
                    "export_name is empty for endpoint {} {}",
                    endpoint.method, endpoint.path
                ));
            }
        }

        Ok(())
    }
}

/// Default query/mutation classification based on HTTP method.
/// - GET requests -> supports_query = true
/// - Other methods -> supports_mutation = true
///
/// This pass can be replaced or extended with custom classification logic
/// by implementing a custom TransformPass.
pub struct DefaultQueryMutationPass;

impl TransformPass for DefaultQueryMutationPass {
    fn name(&self) -> &'static str {
        "default-query-mutation"
    }

    fn apply(&self, input: &mut GeneratorInput) -> Result<(), String> {
        for endpoint in &mut input.endpoints {
            if endpoint.method.eq_ignore_ascii_case("GET") {
                endpoint.supports_query = true;
                endpoint.supports_mutation = false;
            } else {
                endpoint.supports_query = false;
                endpoint.supports_mutation = true;
            }
        }

        Ok(())
    }
}
