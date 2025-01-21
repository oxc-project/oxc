use super::defs::TypeDef;

pub type FileId = usize;
pub type TypeId = usize;

#[expect(dead_code)]
#[derive(Debug)]
pub struct Schema {
    pub defs: Vec<TypeDef>,
    pub files: Vec<File>,
}

#[derive(Debug)]
pub struct File {
    #[expect(dead_code)]
    pub file_path: String,
    pub import_path: String,
}
