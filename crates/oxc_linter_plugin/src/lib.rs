mod errors;
mod plugin;
mod raw_diagnostic;
#[cfg(test)]
mod spans;
#[cfg(test)]
mod test;

pub use plugin::LinterPlugin;
