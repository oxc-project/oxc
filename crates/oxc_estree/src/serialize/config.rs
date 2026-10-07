/// Trait for configs for AST serialization.
pub trait Config {
    /// Enables node comment fields and sparse ownership fixups.
    const COMMENTS: bool = false;

    /// Take ownership records for one native node.
    fn take_comments(&mut self, _node_id: u32) -> Option<Vec<CommentRecord>> {
        None
    }

    /// Buffer containing comment fixups, only present in the comments-enabled config.
    fn comment_fixes(&mut self) -> Option<&mut oxc_data_structures::code_buffer::CodeBuffer> {
        None
    }
    fn new(include_ts_fields: bool, ranges: bool) -> Self;

    /// `true` if output should contain TS fields.
    fn include_ts_fields(&self) -> bool;

    /// `true` if should record paths to `Literal` nodes that need fixing on JS side.
    fn fixes(&self) -> bool;

    /// Get whether output should contain `range` fields.
    fn ranges(&self) -> bool;
}

/// Config for serializing AST without fixes.
pub struct ConfigNoFixes {
    include_ts_fields: bool,
    ranges: bool,
}

impl Config for ConfigNoFixes {
    #[inline(always)]
    fn new(include_ts_fields: bool, ranges: bool) -> Self {
        Self { include_ts_fields, ranges }
    }

    #[inline(always)]
    fn include_ts_fields(&self) -> bool {
        self.include_ts_fields
    }

    #[inline(always)]
    fn fixes(&self) -> bool {
        false
    }

    #[inline(always)]
    fn ranges(&self) -> bool {
        self.ranges
    }
}

/// Config for serializing AST with fixes.
pub struct ConfigFixes {
    include_ts_fields: bool,
    ranges: bool,
}

impl Config for ConfigFixes {
    #[inline(always)]
    fn new(include_ts_fields: bool, ranges: bool) -> Self {
        Self { include_ts_fields, ranges }
    }

    #[inline(always)]
    fn include_ts_fields(&self) -> bool {
        self.include_ts_fields
    }

    #[inline(always)]
    fn fixes(&self) -> bool {
        true
    }

    #[inline(always)]
    fn ranges(&self) -> bool {
        self.ranges
    }
}

/// One source comment's ownership and formatting metadata.
/// The index refers to the parser's flat comment list, including any hashbang prefix.
#[derive(Clone, Copy)]
pub struct CommentRecord {
    /// Native owner ID.
    pub node_id: u32,
    /// Index in the flat source comment list.
    pub index: u32,
    /// Native placement: leading, trailing, or dangling.
    pub placement: u8,
    /// Original comment kind, including HTML and multiline block forms.
    pub kind: u8,
    /// Newline flags before and after the comment.
    pub newlines: u8,
    /// Original annotation classification.
    pub content: u8,
}

/// Compact JSON with literal and comment ownership fixups.
pub struct ConfigCommentFixes {
    base: ConfigFixes,
    owners: rustc_hash::FxHashMap<u32, Vec<CommentRecord>>,
    fixes: oxc_data_structures::code_buffer::CodeBuffer,
}

impl ConfigCommentFixes {
    pub(super) fn set_comments(&mut self, comments: impl Iterator<Item = CommentRecord>) {
        for comment in comments {
            self.owners.entry(comment.node_id).or_default().push(comment);
        }
    }
}

impl Config for ConfigCommentFixes {
    const COMMENTS: bool = true;

    fn new(include_ts_fields: bool, ranges: bool) -> Self {
        Self {
            base: ConfigFixes::new(include_ts_fields, ranges),
            owners: rustc_hash::FxHashMap::default(),
            fixes: oxc_data_structures::code_buffer::CodeBuffer::new(),
        }
    }

    #[inline(always)]
    fn include_ts_fields(&self) -> bool {
        self.base.include_ts_fields()
    }
    #[inline(always)]
    fn fixes(&self) -> bool {
        true
    }
    #[inline(always)]
    fn ranges(&self) -> bool {
        self.base.ranges()
    }
    fn take_comments(&mut self, node_id: u32) -> Option<Vec<CommentRecord>> {
        self.owners.remove(&node_id)
    }
    fn comment_fixes(&mut self) -> Option<&mut oxc_data_structures::code_buffer::CodeBuffer> {
        Some(&mut self.fixes)
    }
}
