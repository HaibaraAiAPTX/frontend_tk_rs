//! barrel:gen command for generating barrel index.ts files.
//!
//! Scans a directory and generates barrel index.ts files for all subdirectories.

use std::path::Path;

use clap::Parser;
use swagger_gen::pipeline::{generate_barrel_for_directory, PlannedFile};
use swagger_tk::model::OpenAPIObject;

/// Options for the barrel:gen command
#[derive(Debug, Clone, Parser)]
pub struct BarrelGenOps {
    /// Input directory to scan for .ts files
    #[arg(short, long)]
    input: String,
}

/// Run barrel:gen command - generate barrel index.ts files for existing TypeScript files
pub fn run_barrel_gen(args: &[String], _open_api: &OpenAPIObject) {
    let result = (|| -> Result<(), String> {
        let args: Vec<String> = std::iter::once("--".to_string())
            .chain(args.iter().cloned())
            .collect();
        let options =
            BarrelGenOps::try_parse_from(args).map_err(|e| format!("Invalid arguments: {e}"))?;

        let input_dir = Path::new(&options.input);

        if !input_dir.exists() {
            return Err(format!("Input directory does not exist: {}", options.input));
        }

        if !input_dir.is_dir() {
            return Err(format!("Input path is not a directory: {}", options.input));
        }

        // Generate barrel files
        let planned_files: Vec<PlannedFile> = generate_barrel_for_directory(&options.input);

        let count = &planned_files.len();

        // Write the generated barrel files
        for planned_file in planned_files {
            let file_path = input_dir.join(&planned_file.path);
            // Skip if the file is just an index.ts we're generating (it would create itself)
            if planned_file.path == "index.ts" && file_path.exists() {
                // Read existing content
                let existing = std::fs::read_to_string(&file_path).unwrap_or_default();
                // Only update if content is different (in case it's a file we're reading from)
                if existing != planned_file.content && !existing.is_empty() {
                    // Don't overwrite existing index.ts with empty content
                    continue;
                }
            }

            // Ensure parent directory exists
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
            }

            std::fs::write(&file_path, &planned_file.content)
                .map_err(|e| format!("Failed to write file {}: {}", planned_file.path, e))?;

            println!("Generated: {}", planned_file.path);
        }

        println!("Barrel generation complete. Generated {} files.", count);                                                               

        Ok(())
    })();

    if let Err(e) = result {
        panic!("barrel:gen failed: {e}");
    }
}
