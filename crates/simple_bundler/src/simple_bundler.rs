use crate::{import_visitor::ImportVisitor, resolver::Resolver};
use oxc::{allocator::Allocator, ast::VisitMut, parser::Parser, span::SourceType};
use oxc_codegen::CodeGenerator;
use oxc_resolver::Resolution;
use oxc_semantic::SemanticBuilder;
use oxc_transformer::{TransformOptions, Transformer};
use sha3::{Digest, Sha3_256};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};

pub struct SimpleBundler {
    resolver: Resolver,
    count: Rc<RefCell<u32>>,
    first_entry: RefCell<Option<String>>,
    /// 已处理的模版列表
    pub processed_modules: RefCell<HashSet<PathBuf>>,
    pub module_map: RefCell<HashMap<String, String>>,
    pub resolution_name_map: Rc<RefCell<HashMap<String, String>>>,
}

impl SimpleBundler {
    pub fn new() -> Self {
        Self {
            count: Rc::new(RefCell::new(0)),
            resolver: Resolver::new(),
            first_entry: RefCell::new(None),
            processed_modules: RefCell::new(HashSet::new()),
            module_map: Default::default(),
            resolution_name_map: Default::default(),
        }
    }

    pub(crate) fn get_resolution(&self, entry: &PathBuf) -> Resolution {
        let dir = {
            if entry.is_absolute() {
                entry.parent().unwrap().to_path_buf()
            } else {
                PathBuf::from(self.first_entry.borrow().as_ref().unwrap())
                    .parent()
                    .unwrap()
                    .to_path_buf()
            }
        };
        self.resolver.resolve(&dir, &entry).unwrap()
    }

    pub fn bundle(&self, entry: &PathBuf) -> String {
        self.update_first_entry(entry);
        let resolution = self.get_resolution(&entry);
        let full_path = resolution.full_path();
        if self.processed_modules.borrow().contains(&full_path) {
            return self
                .resolution_name_map
                .borrow()
                .get(full_path.to_str().unwrap())
                .expect(&format!(
                    "entry not found: {:?}, {:#?}",
                    &full_path, &self.resolution_name_map
                ))
                .clone();
        }
        let file_name = self._bundle(entry, &resolution.path());
        self.processed_modules.borrow_mut().insert(full_path);
        format!("{}.js", file_name)
    }

    fn _bundle(&self, entry: &PathBuf, resolution: &Path) -> String {
        let allocator = Allocator::default();
        let source_text = fs::read_to_string(&resolution).unwrap();
        let source_type = SourceType::from_path(&resolution).unwrap();
        let ret = Parser::new(&allocator, &source_text, source_type).parse();
        if !ret.errors.is_empty() {
            println!("Parser Errors:");
            for error in ret.errors {
                let error = error.with_source_code(source_text.clone());
                println!("{error:?}");
            }
        }

        let mut program = ret.program;

        let dir: PathBuf = resolution.parent().unwrap().to_path_buf();
        *self.count.borrow_mut() += 1;
        let mut import_visitor = ImportVisitor {
            id: *self.count.borrow(),
            dir: &dir,
            bundler: self,
            entry,
            allocator: &allocator,
        };
        import_visitor.visit_program(&mut program);

        let ret = SemanticBuilder::new()
            .with_excess_capacity(2.0)
            .build(&program);

        if !ret.errors.is_empty() {
            println!("Semantic Errors:");
            for error in ret.errors {
                let error = error.with_source_code(source_text.clone());
                println!("{error:?}");
            }
        }

        let (symbols, scopes) = ret.semantic.into_symbol_table_and_scope_tree();

        let transform_options = TransformOptions::default();

        let ret = Transformer::new(&allocator, resolution, transform_options)
            .build_with_symbols_and_scopes(symbols, scopes, &mut program);

        if !ret.errors.is_empty() {
            println!("Transformer Errors:");
            for error in ret.errors {
                let error = error.with_source_code(source_text.clone());
                println!("{error:?}");
            }
        }

        let printed = CodeGenerator::new().build(&program).code;
        let file_name = self.cache_transformed(resolution, printed);
        self.resolution_name_map
            .borrow_mut()
            .insert(resolution.to_str().unwrap().to_string(), file_name.clone());
        file_name
    }

    fn cache_transformed(&self, path: &Path, transformed_code: String) -> String {
        let file_name = path.file_stem().expect("获取文件名失败").to_str().unwrap();
        let mut hasher = Sha3_256::new();
        hasher.update(transformed_code.as_bytes());
        let hash: [u8; 32] = hasher.finalize().into();
        let hex_str = hex::encode(&hash[..8]);
        let new_file_name = format!("{}.{}", file_name, hex_str);
        self.module_map
            .borrow_mut()
            .insert(new_file_name.clone(), transformed_code);
        new_file_name
    }

    pub fn write(&self, path: &PathBuf) {
        if !path.exists() {
            fs::create_dir_all(path).unwrap();
        }
        self.module_map.borrow().iter().for_each(|(k, v)| {
            let p = path.join(format!("{k}.js"));
            fs::write(p, v).unwrap();
        });
    }

    fn update_first_entry(&self, entry: &PathBuf) {
        if self.first_entry.borrow().is_none() {
            let file_name = entry.to_str().unwrap().to_string();
            *self.first_entry.borrow_mut() = Some(file_name);
        }
    }
}
