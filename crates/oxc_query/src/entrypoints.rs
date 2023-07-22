use trustfall::provider::{ResolveInfo, VertexIterator};

use super::vertex::Vertex;

pub(super) fn file<'a, 'b: 'a>(_resolve_info: &ResolveInfo) -> VertexIterator<'a, Vertex<'b>> {
    todo!("implement resolving starting vertices for entrypoint edge 'File'")
}
