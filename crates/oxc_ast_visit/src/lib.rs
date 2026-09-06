mod generated {
    #[cfg(feature = "serialize")]
    mod utf8_to_utf16_converter;
    pub mod visit;
    pub mod visit_js;
    pub mod visit_js_mut;
    pub mod visit_mut;
}

mod comment_attachment;
mod node_id;

pub use comment_attachment::{
    AttachedComment, CommentAttachmentBuilder, CommentAttachmentCollector, CommentAttachments,
    CommentPlacement,
};
pub use generated::{visit::*, visit_js::*, visit_js_mut::*, visit_mut::*};
pub use node_id::AstNodeIdAssigner;

#[cfg(feature = "serialize")]
pub mod utf8_to_utf16;
