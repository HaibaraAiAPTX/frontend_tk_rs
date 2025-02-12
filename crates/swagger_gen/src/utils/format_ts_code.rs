use std::path::PathBuf;
use dprint_plugin_typescript::{configuration::ConfigurationBuilder, format_text};

pub fn format_ts_code(code: &str) -> Result<String, String> {
    let mut config = ConfigurationBuilder::new();
    config.line_width(80);
    config.use_tabs(false);
    config.indent_width(4);
    config.semi_colons(dprint_plugin_typescript::configuration::SemiColons::Asi);

    let formatted_code = format_text(
        &PathBuf::from("test.ts"),
        Some("ts"),
        code.to_string(),
        &config.build(),
    );

    match formatted_code {
        Ok(Some(code)) => Ok(code),
        Ok(None) => Err("The code was already formatted. ".to_string()),
        Err(e) => Err(e.to_string()),
    }
}
