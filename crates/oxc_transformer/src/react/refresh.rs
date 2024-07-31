use super::options::ReactRefreshOptions;
use std::rc::Rc;

use crate::context::Ctx;

/// React Fast Refresh
///
/// Transform React components to integrate Fast Refresh.
///
/// References:
///
/// * <https://github.com/facebook/react/issues/16604#issuecomment-528663101>
/// * <https://github.com/facebook/react/blob/main/packages/react-refresh/src/ReactFreshBabelPlugin.js>
pub struct ReactRefresh<'a> {
    options: ReactRefreshOptions,
    ctx: Ctx<'a>,
}

impl<'a> ReactRefresh<'a> {
    pub fn new(options: ReactRefreshOptions, ctx: Ctx<'a>) -> Self {
        Self { options, ctx }
    }
}
