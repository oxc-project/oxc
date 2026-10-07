use smallvec::SmallVec;

use oxc_ast::{Comment, CommentAttachment};
use oxc_syntax::node::NodeId;

struct OwnerRewrite<'a> {
    old_id: NodeId,
    new_id: NodeId,
    attachment: &'a CommentAttachment,
}

/// A sparse old-to-new ID lookup containing only comments with surviving owner
/// candidates. Ordered old IDs advance a cursor once through the records;
/// construction-order IDs and moved nodes can search without rewinding it.
///
/// For example, comments owned by IDs 10 and 100 need two records even if the
/// transformed AST contains thousands of nodes. Visiting 100 before 10 uses the
/// forward cursor for 100, then binary search for 10.
pub struct CommentIdRemapper<'a> {
    records: SmallVec<[OwnerRewrite<'a>; 8]>,
    cursor: usize,
    high_water: NodeId,
    first_unmapped: usize,
    minimum_old_id: NodeId,
    // Stop reading old AST IDs once every candidate has been found. Missing
    // owners keep this nonzero so a later reordered visit can still claim them.
    remaining: usize,
}

impl<'a> CommentIdRemapper<'a> {
    pub(crate) fn new(comments: &'a [Comment]) -> Option<Self> {
        let mut owners = comments.iter().filter_map(|comment| {
            let attachment = comment.attachment.as_ref()?;
            let old_id = attachment.node_id.get();
            if old_id == NodeId::ORPHANED {
                return None;
            }
            Some(OwnerRewrite {
                old_id,
                attachment,
                // Missing owners remain orphaned, including across another
                // rebuild. Their old IDs cannot collide with new dense IDs.
                new_id: NodeId::ORPHANED,
            })
        });
        // Reserve once, but only if at least one comment has a candidate owner.
        // `filter_map` cannot provide a lower size hint for `SmallVec::collect`.
        let first = owners.next()?;
        let mut records: SmallVec<[_; 8]> = SmallVec::with_capacity(comments.len());
        records.push(first);
        records.extend(owners);
        records.sort_unstable_by_key(|record| record.old_id);
        let remaining = records.len();
        let minimum_old_id = records[0].old_id;
        Some(Self {
            records,
            cursor: 0,
            high_water: NodeId::ROOT,
            first_unmapped: 0,
            minimum_old_id,
            remaining,
        })
    }

    #[inline]
    pub(crate) fn minimum_old_id(&self) -> Option<NodeId> {
        (self.remaining != 0).then_some(self.minimum_old_id)
    }

    #[inline]
    pub(crate) fn enter_node(&mut self, old_id: NodeId, new_id: NodeId) {
        // Parser parents are often constructed after their descendants. Avoid
        // binary searches for descendants below every still-unclaimed owner.
        if old_id < self.minimum_old_id {
            return;
        }
        // Generated nodes use DUMMY, which shares ROOT's value. Only Program can
        // claim that owner; synthetic descendants must not steal file comments.
        if old_id == NodeId::DUMMY && new_id != NodeId::ROOT {
            return;
        }
        if old_id >= self.high_water {
            let records = self.records.as_mut_slice();
            if self.cursor == records.len() || records[self.cursor].old_id > old_id {
                return;
            }
            self.high_water = old_id;
            while self.cursor < records.len() && records[self.cursor].old_id < old_id {
                self.cursor += 1;
            }
            while self.cursor < records.len() && records[self.cursor].old_id == old_id {
                records[self.cursor].new_id = new_id;
                self.cursor += 1;
                self.remaining -= 1;
            }
        } else {
            self.enter_reordered_node(old_id, new_id);
        }
        if old_id == self.minimum_old_id {
            self.advance_minimum();
        }
    }

    #[inline(never)]
    fn advance_minimum(&mut self) {
        // The forward cursor can skip owners which a later reordered visit
        // still needs. Advance this minimum only past claimed records.
        while self.first_unmapped < self.records.len()
            && self.records[self.first_unmapped].new_id != NodeId::ORPHANED
        {
            self.first_unmapped += 1;
        }
        self.minimum_old_id =
            self.records.get(self.first_unmapped).map_or(NodeId::ORPHANED, |record| record.old_id);
    }

    #[inline(never)]
    fn enter_reordered_node(&mut self, old_id: NodeId, new_id: NodeId) {
        let records = self.records.as_mut_slice();
        let start = records.partition_point(|record| record.old_id < old_id);
        for record in &mut records[start..] {
            // Equal-ID records are rewritten together. Once the first is
            // claimed, every comment in this owner group is already mapped.
            // Avoid rescanning the group for clones retaining semantic IDs.
            if record.old_id != old_id || record.new_id != NodeId::ORPHANED {
                break;
            }
            // A clone retaining semantic IDs can share an old ID. Match the
            // first surviving visit, consistent with codegen claiming an owner once.
            record.new_id = new_id;
            self.remaining -= 1;
        }
    }

    pub(crate) fn finish(self) {
        for record in self.records {
            record.attachment.node_id.set(record.new_id);
        }
    }
}
