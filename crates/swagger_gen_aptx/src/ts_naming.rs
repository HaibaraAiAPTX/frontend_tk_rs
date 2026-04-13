use std::collections::HashMap;

use inflector::cases::{camelcase::to_camel_case, pascalcase::to_pascal_case};

use swagger_gen::pipeline::EndpointItem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedTsName {
    pub(crate) file_stem: String,
    pub(crate) export_name: String,
    pub(crate) builder_name: String,
}

pub(crate) fn resolve_final_ts_names(endpoints: &[EndpointItem]) -> Vec<ResolvedTsName> {
    let namespace_prefixes = resolve_namespace_common_prefixes(endpoints);
    let planned: Vec<(String, String, String, String)> = endpoints
        .iter()
        .map(|endpoint| {
            let namespace_path = get_namespace_path(endpoint);
            let common_prefix = namespace_prefixes
                .get(&namespace_path)
                .map(|s| s.as_str())
                .unwrap_or("");
            (
                namespace_path,
                compute_ts_name(endpoint, common_prefix),
                compute_fallback_ts_name(endpoint, common_prefix),
                endpoint.method.clone(),
            )
        })
        .collect();

    let mut short_name_counts: HashMap<(String, String), usize> = HashMap::new();
    for (namespace_path, short_name, _, _) in &planned {
        *short_name_counts
            .entry((namespace_path.clone(), short_name.clone()))
            .or_insert(0) += 1;
    }

    let mut used_counts: HashMap<(String, String), usize> = HashMap::new();

    planned
        .into_iter()
        .map(|(namespace_path, short_name, fallback_name, method)| {
            let prefers_short = short_name_counts
                .get(&(namespace_path.clone(), short_name.clone()))
                .copied()
                .unwrap_or(0)
                == 1;

            let mut final_name = if prefers_short {
                short_name
            } else {
                fallback_name
            };

            if name_used(&used_counts, &namespace_path, &final_name) {
                final_name = sanitize_reserved(&normalize_identifier(format!(
                    "{}{}",
                    final_name,
                    to_pascal_case(&method.to_lowercase())
                )));
            }

            if name_used(&used_counts, &namespace_path, &final_name) {
                let mut serial = 2usize;
                let mut serial_candidate = format!("{final_name}_{serial}");
                while name_used(&used_counts, &namespace_path, &serial_candidate) {
                    serial += 1;
                    serial_candidate = format!("{final_name}_{serial}");
                }
                final_name = serial_candidate;
            }

            *used_counts
                .entry((namespace_path, final_name.clone()))
                .or_insert(0) += 1;

            ResolvedTsName {
                file_stem: final_name.clone(),
                export_name: final_name.clone(),
                builder_name: format!("build{}Spec", to_pascal_case(&final_name)),
            }
        })
        .collect()
}

fn name_used(
    used_counts: &HashMap<(String, String), usize>,
    namespace_path: &str,
    candidate: &str,
) -> bool {
    used_counts
        .get(&(namespace_path.to_string(), candidate.to_string()))
        .copied()
        .unwrap_or(0)
        > 0
}

fn extract_service_part(name: &str) -> &str {
    let idx = name.find(|c: char| c.is_uppercase()).unwrap_or(0);
    &name[idx..]
}

fn find_common_service_prefix(endpoints: &[&EndpointItem]) -> String {
    if endpoints.len() <= 1 {
        return String::new();
    }

    let parts: Vec<&str> = endpoints
        .iter()
        .map(|endpoint| extract_service_part(&endpoint.operation_name))
        .collect();

    let mut prefix_words = split_identifier_words(parts[0]);
    for part in &parts[1..] {
        let words = split_identifier_words(part);
        let common_len = prefix_words
            .iter()
            .zip(words.iter())
            .take_while(|(left, right)| left == right)
            .count();
        prefix_words.truncate(common_len);
        if prefix_words.is_empty() {
            return String::new();
        }
    }

    let namespace_words = split_identifier_words(&namespace_to_camel(&endpoints[0].namespace));
    if let Some(index) = find_word_sequence(&prefix_words, &namespace_words) {
        prefix_words.truncate(index);
        if prefix_words.is_empty() {
            return String::new();
        }
    }

    let prefix = prefix_words.join("");
    if parts
        .iter()
        .any(|part| part.strip_prefix(&prefix).unwrap_or("").is_empty())
    {
        return String::new();
    }

    prefix
}

fn resolve_namespace_common_prefixes(endpoints: &[EndpointItem]) -> HashMap<String, String> {
    let mut grouped: HashMap<String, Vec<&EndpointItem>> = HashMap::new();
    for endpoint in endpoints {
        grouped
            .entry(get_namespace_path(endpoint))
            .or_default()
            .push(endpoint);
    }

    grouped
        .into_iter()
        .map(|(namespace_path, group)| (namespace_path, find_common_service_prefix(&group)))
        .collect()
}

fn compute_ts_name(endpoint: &EndpointItem, common_prefix: &str) -> String {
    let service_part = extract_service_part(&endpoint.operation_name);
    let after_service = service_part.strip_prefix(common_prefix).unwrap_or(service_part);
    let namespace_camel = namespace_to_camel(&endpoint.namespace);

    let action = if let Some(rest) = after_service.strip_prefix(&namespace_camel) {
        rest
    } else if let Some(index) = after_service.find(&namespace_camel) {
        &after_service[index + namespace_camel.len()..]
    } else {
        after_service
    };

    if action.is_empty() {
        return sanitize_reserved(&normalize_identifier(to_camel_case(after_service)));
    }

    sanitize_reserved(&normalize_identifier(to_camel_case(action)))
}

fn compute_fallback_ts_name(endpoint: &EndpointItem, common_prefix: &str) -> String {
    let service_part = extract_service_part(&endpoint.operation_name);
    let after_service = service_part.strip_prefix(common_prefix).unwrap_or(service_part);
    sanitize_reserved(&normalize_identifier(to_camel_case(after_service)))
}

fn get_namespace_path(endpoint: &EndpointItem) -> String {
    endpoint.namespace.join("/")
}

fn namespace_to_camel(namespace: &[String]) -> String {
    namespace
        .iter()
        .flat_map(|segment| segment.split(['-', '_']))
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
            }
        })
        .collect()
}

fn split_identifier_words(name: &str) -> Vec<String> {
    let chars: Vec<char> = name.chars().collect();
    if chars.is_empty() {
        return vec![];
    }

    let mut words = Vec::new();
    let mut current = String::new();

    for (index, ch) in chars.iter().enumerate() {
        let starts_new_word = if index == 0 {
            false
        } else if ch.is_uppercase() {
            let prev = chars[index - 1];
            let next_is_lower = chars.get(index + 1).is_some_and(|c| c.is_lowercase());
            prev.is_lowercase() || (prev.is_uppercase() && next_is_lower)
        } else {
            false
        };

        if starts_new_word && !current.is_empty() {
            words.push(current);
            current = String::new();
        }

        current.push(*ch);
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

fn find_word_sequence(haystack: &[String], needle: &[String]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }

    haystack.windows(needle.len()).position(|window| {
        window
            .iter()
            .zip(needle.iter())
            .all(|(left, right)| left == right)
    })
}

fn normalize_identifier(mut value: String) -> String {
    if value.trim().is_empty() {
        return "op".to_string();
    }

    if value.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        value = format!("op{}", to_pascal_case(&value));
    }

    value
}

fn sanitize_reserved(value: &str) -> String {
    const RESERVED: [&str; 12] = [
        "delete", "default", "class", "function", "new", "return", "switch", "case", "var",
        "let", "const", "import",
    ];

    if RESERVED.contains(&value) {
        format!("do{}", to_pascal_case(value))
    } else {
        value.to_string()
    }
}
