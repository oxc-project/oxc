use crate::LintContext;

pub fn is_in_app_dir(file_path: &str) -> bool {
    file_path.contains("app/") || file_path.contains("app\\")
}

pub fn is_document_page(file_path: &str) -> bool {
    let Some(page) = file_path.split("pages").last() else {
        return false;
    };
    page.starts_with("/_document") || page.starts_with("\\_document")
}

pub fn get_next_script_import_local_name<'a>(ctx: &'a LintContext) -> Option<&'a str> {
    ctx.module_record().import_entries.iter().find_map(|entry| {
        if entry.module_request.name() == "next/script" {
            Some(entry.local_name.name())
        } else {
            None
        }
    })
}
