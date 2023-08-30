mod errors;
mod js;
mod plugin;
mod raw_diagnostic;
#[cfg(test)]
mod spans;
#[cfg(test)]
mod test;
mod util;

pub use {plugin::LinterPlugin, util::make_relative_path_parts};
