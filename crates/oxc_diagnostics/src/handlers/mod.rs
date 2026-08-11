/*!
Renderers included with `oxc_diagnostics`.
*/

pub use graphical::*;
pub use json::*;
pub use theme::GraphicalTheme;

mod graphical;
mod json;
#[expect(clippy::redundant_pub_crate, reason = "prevents public glob re-export")]
pub(crate) mod theme;
