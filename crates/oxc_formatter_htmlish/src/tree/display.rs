/// The CSS `display` an element gets from the UA stylesheet (Prettier's table, `constants.evaluate.js`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Display {
    #[default]
    Inline,
    Block,
    InlineBlock,
    ListItem,
    TableCell,
    /// Every other `table*` value.
    Table,
    None,
    /// `contents`, `ruby`, or a value from a `<!-- display: -->` comment.
    Other,
}

impl Display {
    fn block_like(self) -> bool {
        matches!(self, Self::Block | Self::ListItem | Self::Table | Self::TableCell)
    }

    /// Whitespace at the start of the first child / the end of the last child / of an empty element is significant.
    pub fn edges_sensitive(self) -> bool {
        !self.block_like() && self != Self::InlineBlock
    }

    /// Whitespace between this and a sibling is significant.
    pub fn sides_sensitive(self) -> bool {
        !self.block_like()
    }
}

/// `<!-- display: block -->` (Prettier's `/^\s*display:\s*([a-z]+)\s*$/`).
pub fn display_comment(content: &str) -> Option<Display> {
    let value = content.trim().strip_prefix("display:")?.trim_start();
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_lowercase()) {
        return None;
    }
    Some(match value {
        "block" => Display::Block,
        "inline" => Display::Inline,
        "table" => Display::Table,
        "none" => Display::None,
        _ => Display::Other,
    })
}

/// The UA stylesheet's `display` for an HTML element, as Prettier tabulates it.
pub fn ua_display(tag: &str) -> Display {
    match tag {
        // `script` as the UA has it;
        // Prettier lays it out as a block (DIVERGENCES.md#script-display-none).
        "area" | "base" | "basefont" | "datalist" | "head" | "link" | "meta" | "noembed"
        | "noframes" | "rp" | "script" | "style" | "title" => Display::None,
        "param" | "html" | "body" | "address" | "blockquote" | "center" | "div" | "figure"
        | "figcaption" | "footer" | "form" | "header" | "hr" | "legend" | "listing" | "main"
        | "p" | "plaintext" | "pre" | "search" | "xmp" | "article" | "aside" | "h1" | "h2"
        | "h3" | "h4" | "h5" | "h6" | "hgroup" | "nav" | "section" | "dir" | "dd" | "dl" | "dt"
        | "menu" | "ol" | "ul" | "fieldset" | "details" | "summary" | "option" | "optgroup"
        | "source" | "track" => Display::Block,
        "slot" | "ruby" | "rt" => Display::Other,
        "li" => Display::ListItem,
        "table" | "caption" | "colgroup" | "col" | "thead" | "tbody" | "tfoot" | "tr" => {
            Display::Table
        }
        "td" | "th" => Display::TableCell,
        // `dialog`: out of flow (`display: none` closed, `position: absolute` open), with a box for its content;
        // Prettier lays it out as a block (DIVERGENCES.md#dialog-out-of-flow).
        "input" | "button" | "marquee" | "select" | "meter" | "progress" | "object" | "video"
        | "audio" | "dialog" => Display::InlineBlock,
        _ => Display::Inline,
    }
}
