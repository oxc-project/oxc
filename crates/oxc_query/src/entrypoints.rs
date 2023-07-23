use trustfall::provider::{ResolveInfo, VertexIterator};

use super::vertex::Vertex;

pub(super) fn file<'a, 'b: 'a>(_resolve_info: &ResolveInfo) -> VertexIterator<'a, Vertex<'b>> {
    Box::new(std::iter::once(Vertex::File))
}
