use std::path::Path;

use crate::plugin::test_queries;

#[test]
fn query_tests() -> oxc_diagnostics::Result<()> {
    test_queries(&Path::new("examples/queries").to_path_buf())?;
    Ok(())
}
