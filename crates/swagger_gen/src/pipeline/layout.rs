use super::model::PlannedFile;

pub trait LayoutStrategy {
    fn id(&self) -> &'static str;
    fn apply(&self, files: Vec<PlannedFile>) -> Vec<PlannedFile>;
}

#[derive(Default)]
pub struct IdentityLayout;

impl LayoutStrategy for IdentityLayout {
    fn id(&self) -> &'static str {
        "identity"
    }

    fn apply(&self, files: Vec<PlannedFile>) -> Vec<PlannedFile> {
        files
    }
}
