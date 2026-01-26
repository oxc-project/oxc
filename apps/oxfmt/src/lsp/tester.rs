use tower_lsp_server::ls_types::Uri;

use crate::lsp::server_formatter::{ServerFormatter, ServerFormatterBuilder};
use oxc_language_server::{Tool, ToolRestartChanges};

/// Testing struct for the [formatter server][crate::formatter::server_formatter::ServerFormatter].
pub struct Tester<'t> {
    relative_root_dir: &'t str,
    options: serde_json::Value,
}

impl Tester<'_> {
    pub fn new(relative_root_dir: &'static str, options: serde_json::Value) -> Self {
        Self { relative_root_dir, options }
    }

    fn create_formatter(&self) -> ServerFormatter {
        ServerFormatterBuilder::dummy()
            .build(&Self::get_root_uri(self.relative_root_dir), self.options.clone())
    }

    pub fn get_root_uri(relative_root_dir: &str) -> Uri {
        let absolute_path =
            std::env::current_dir().expect("could not get current dir").join(relative_root_dir);

        Uri::from_file_path(absolute_path).expect("could not convert current dir to uri")
    }

    pub fn handle_configuration_change(
        &self,
        new_options: serde_json::Value,
    ) -> ToolRestartChanges {
        let builder = ServerFormatterBuilder::dummy();
        self.create_formatter().handle_configuration_change(
            &builder,
            &Self::get_root_uri(self.relative_root_dir),
            &self.options,
            new_options,
        )
    }
}
