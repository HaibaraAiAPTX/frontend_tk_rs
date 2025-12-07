use crate::SimpleBundler;
use oxc::{
    allocator::{Allocator, FromIn},
    ast::ast::{ExportNamedDeclaration, ImportDeclaration},
};
use oxc_ast_visit::VisitMut;
use oxc_resolver::Resolution;
use oxc_span::Atom;
use path_clean::PathClean;
use std::path::PathBuf;

pub struct ImportVisitor<'a, 'b>
where
    'b: 'a,
{
    pub id: u32,
    pub dir: &'a PathBuf,
    pub bundler: &'b SimpleBundler,
    pub entry: &'a PathBuf,
    pub allocator: &'b Allocator,
}

impl<'a, 'b> ImportVisitor<'a, 'b> {
    fn should_bundle(&self, entry: &PathBuf) -> (bool, Resolution) {
        let resolution = self.bundler.get_resolution(entry);
        let full_path = resolution.full_path();
        (
            !self.bundler.processed_modules.borrow().contains(&full_path),
            resolution,
        )
    }

    fn get_entry(&self, source_path: &str) -> PathBuf {
        if source_path.starts_with('.') {
            PathBuf::from(&self.dir).join(source_path).clean()
        } else {
            PathBuf::from(source_path)
        }
    }

    fn start_bundle(&self, entry: &PathBuf, mut f: impl FnMut(String)) {
        let (should_bundle, resolution) = self.should_bundle(entry);
        if should_bundle {
            self.bundler.bundle(entry);
        }
        let resolution_path = resolution.full_path().to_str().unwrap().to_string();
        let file_name = self
            .bundler
            .resolution_name_map
            .borrow()
            .get(&resolution_path)
            .unwrap_or_else(|| {
                panic!(
                    "entry not found: {:?}, {:#?}",
                    &resolution_path, &self.bundler.resolution_name_map
                )
            })
            .clone();
        let new_source = format!("./{file_name}.js");
        f(new_source);
    }
}

impl<'a, 'b> VisitMut<'a> for ImportVisitor<'a, 'b>
where
    'b: 'a,
{
    fn visit_import_declaration(&mut self, it: &mut ImportDeclaration<'a>) {
        let source_path = it.source.value.to_string();
        let entry = self.get_entry(&source_path);
        self.start_bundle(&entry, |new_source| {
            it.source.value = Atom::from_in(&new_source, self.allocator);
        });
    }

    // fn visit_export_default_declaration(&mut self, it: &mut ExportDefaultDeclaration<'a>) {
    //     let source_path = it.exported.identifier_name().unwrap().to_string();
    //     dbg!(&source_path);
    // }

    // fn visit_export_all_declaration(&mut self, it: &mut ExportAllDeclaration<'a>) {
    //     let source_path = it.source.value.to_string();
    //     dbg!(&source_path);
    // }

    fn visit_export_named_declaration(&mut self, it: &mut ExportNamedDeclaration<'a>) {
        if it.source.is_some() {
            let source_path = it.source.as_ref().unwrap().value.to_string();
            let entry = self.get_entry(&source_path);
            self.start_bundle(&entry, |new_source| {
                it.source.as_mut().unwrap().value = Atom::from_in(&new_source, self.allocator);
            });
        }
    }
}
