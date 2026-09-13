//! Semantic fingerprint: what formatting must leave unchanged.
//!
//! Idempotency (`format(format(x)) == format(x)`) only says the output is stable;
//! a corrupted output is often a fixpoint (a list that became a paragraph, an escape that became an entity).
//! This walks the AST of the input and of the output and serializes the MEANING only,
//! so `fingerprint(x) == fingerprint(format(x))` is `parse(format(x)) ≅ parse(x)`.
//!
//! Ignored, because formatting may change them:
//! spans; the style facts this AST records and mdast does not (markers, fence chars, ATX vs setext,
//! break kinds, indented vs fenced); text splitting and whitespace runs; list tightness / spread
//! (Prettier's `massage-ast` drops `spread` / `isAligned` too, and collapses whitespace in labels and inline code).
//!
//! Compared, where Prettier's `massage-ast` looks away:
//! - text content (decoded, whitespace collapsed): Prettier drops `text` nodes, which is where its escaping bugs hide
//! - code block content: Prettier drops `code.value` because it formats the embedded language;
//!   ours stays verbatim until embedding lands (then the content of dispatched languages gets exempted)
//!
//! Every entry is one line; nesting is indentation, so a mismatch reads as a diff.

use std::fmt::Write as _;

use cow_utils::CowUtils;

use oxc_markdown_parser::{
    Segment, Span,
    ast::{
        Block, CodeBlockKind, Inline, LinkKind, ListItem, ListMarker, ReferenceKind, Root,
        TableAlign,
    },
    decode, label,
};

pub fn fingerprint(source: &str, root: &Root<'_>) -> String {
    let mut w = Walker { source, out: String::new(), depth: 0 };
    w.blocks(&root.children);
    w.out
}

struct Walker<'s> {
    source: &'s str,
    out: String,
    depth: usize,
}

impl Walker<'_> {
    fn line(&mut self, s: &str) {
        for _ in 0..self.depth {
            self.out.push_str("  ");
        }
        self.out.push_str(s);
        self.out.push('\n');
    }

    fn nested(&mut self, f: impl FnOnce(&mut Self)) {
        self.depth += 1;
        f(self);
        self.depth -= 1;
    }

    fn raw(&self, span: Span) -> &str {
        span.slice(self.source)
    }

    fn joined(&self, pieces: &[Segment]) -> String {
        Segment::join(self.source, pieces).into_owned()
    }

    fn blocks(&mut self, blocks: &[Block<'_>]) {
        for block in blocks {
            self.block(block);
        }
    }

    fn block(&mut self, block: &Block<'_>) {
        match block {
            Block::Paragraph(p) => {
                let text = self.inlines(&p.children);
                self.line(&format!("paragraph {text}"));
            }
            Block::Heading(h) => {
                let text = self.inlines(&h.children);
                self.line(&format!("heading {} {text}", h.level));
            }
            Block::ThematicBreak(_) => self.line("thematic-break"),
            Block::CodeBlock(c) => {
                // Fenced vs indented is a style fact; the info string and the content are not.
                let info = match c.kind {
                    CodeBlockKind::Fenced { info: Some(info), .. } => self.raw(info).to_string(),
                    _ => String::new(),
                };
                let content = self.joined(&c.lines);
                self.line(&format!("code [{info}] {content:?}"));
            }
            Block::HtmlBlock(h) => {
                let content = self.joined(&h.lines);
                self.line(&format!("html {content:?}"));
            }
            Block::Blockquote(q) => {
                self.line("blockquote");
                self.nested(|w| w.blocks(&q.children));
            }
            Block::List(l) => {
                let head = match l.marker {
                    ListMarker::Bullet { .. } => "list bullet".to_string(),
                    ListMarker::Ordered { .. } => {
                        // Only the start number is semantic; the rest are renumbered freely.
                        let start = l
                            .children
                            .first()
                            .map_or("", |item| self.raw(item.marker).trim_end_matches(['.', ')']));
                        format!("list ordered start={}", start.trim_start_matches('0'))
                    }
                };
                self.line(&head);
                self.nested(|w| {
                    for item in &l.children {
                        w.list_item(item);
                    }
                });
            }
            Block::Definition(d) => {
                let label = label::normalize(&self.joined(&d.label)).into_owned();
                let dest = decode::destination(self.source, &d.destination);
                let title = d.title.as_ref().map(|t| decode::title(self.source, t));
                self.line(&format!("definition [{label}] {dest} {title:?}"));
            }
            Block::Table(t) => {
                let align: Vec<&str> = t
                    .align
                    .iter()
                    .map(|a| match a {
                        TableAlign::None => "-",
                        TableAlign::Left => "l",
                        TableAlign::Center => "c",
                        TableAlign::Right => "r",
                    })
                    .collect();
                self.line(&format!("table {}", align.join("")));
                self.nested(|w| {
                    for row in &t.children {
                        let cells: Vec<String> =
                            row.children.iter().map(|cell| w.inlines(&cell.children)).collect();
                        w.line(&format!("row {cells:?}"));
                    }
                });
            }
            Block::FootnoteDefinition(fd) => {
                let label = label::normalize(self.raw(fd.label)).into_owned();
                self.line(&format!("footnote-definition [{label}]"));
                self.nested(|w| w.blocks(&fd.children));
            }
            Block::MathBlock(m) => {
                let meta = m.meta.map(|s| self.raw(s).to_string());
                let content = self.joined(&m.lines);
                self.line(&format!("math {meta:?} {content:?}"));
            }
            Block::Liquid(l) => {
                let content = self.joined(&l.pieces);
                self.line(&format!("liquid {content:?}"));
            }
            Block::ContainerDirective(d) => {
                let opening = self.raw(d.opening).trim();
                let closed = d.closing.is_some();
                self.line(&format!("directive {opening:?} closed={closed}"));
                self.nested(|w| w.blocks(&d.children));
            }
            Block::MdxEsm(n) => {
                let raw = self.raw(n.span);
                self.line(&format!("mdx-esm {raw:?}"));
            }
            Block::MdxExpression(n) => {
                let raw = self.raw(n.span);
                self.line(&format!("mdx-expression {raw:?}"));
            }
            Block::MdxJsx(j) => {
                let opening = self.raw(j.opening);
                self.line(&format!("mdx-jsx {opening:?}"));
                self.nested(|w| w.blocks(&j.children));
            }
        }
    }

    fn list_item(&mut self, item: &ListItem<'_>) {
        let head = match item.checkbox {
            Some(c) if c.checked => "item [x]",
            Some(_) => "item [ ]",
            None => "item",
        };
        self.line(head);
        self.nested(|w| w.blocks(&item.children));
    }

    /// Phrasing content as one string: texts concatenate, so how they were split does not matter.
    fn inlines(&self, inlines: &[Inline<'_>]) -> String {
        let mut out = String::new();
        for inline in inlines {
            self.inline(inline, &mut out);
        }
        collapse_whitespace(&out)
    }

    fn inline(&self, inline: &Inline<'_>, out: &mut String) {
        match inline {
            Inline::Text(t) => out.push_str(&decode::decode(self.raw(t.span), true)),
            Inline::SoftBreak(_) => out.push(' '),
            Inline::HardBreak(_) => out.push_str("<br>"),
            Inline::Emphasis(e) => {
                out.push_str("<em>");
                for child in &e.children {
                    self.inline(child, out);
                }
                out.push_str("</em>");
            }
            Inline::Strong(s) => {
                out.push_str("<strong>");
                for child in &s.children {
                    self.inline(child, out);
                }
                out.push_str("</strong>");
            }
            Inline::Strikethrough(s) => {
                out.push_str("<del>");
                for child in &s.children {
                    self.inline(child, out);
                }
                out.push_str("</del>");
            }
            Inline::CodeSpan(c) => {
                // Prettier's massage: `\n` inside inline code is a space.
                let content = self.joined(&c.pieces).cow_replace('\n', " ").into_owned();
                let _ = write!(out, "<code>{content}</code>");
            }
            Inline::Link(l) => {
                out.push_str("<a ");
                self.link_kind(&l.kind, out);
                out.push('>');
                for child in &l.children {
                    self.inline(child, out);
                }
                out.push_str("</a>");
            }
            Inline::Image(i) => {
                out.push_str("<img ");
                self.link_kind(&i.kind, out);
                out.push_str(" alt=");
                for child in &i.children {
                    self.inline(child, out);
                }
                out.push('>');
            }
            Inline::Autolink(a) => {
                let _ = write!(out, "<autolink {}>", self.raw(a.span));
            }
            Inline::AutolinkLiteral(a) => out.push_str(self.raw(a.span)),
            Inline::HtmlInline(h) => out.push_str(&self.joined(&h.pieces)),
            Inline::FootnoteReference(r) => {
                let _ = write!(out, "<sup [{}]>", label::normalize(self.raw(r.label)));
            }
            Inline::MathSpan(m) => out.push_str(&self.joined(&m.pieces)),
            Inline::WikiLink(w) => out.push_str(self.raw(w.span)),
            Inline::Liquid(l) => out.push_str(&self.joined(&l.pieces)),
            Inline::MdxExpression(n) => out.push_str(self.raw(n.span)),
            Inline::MdxJsx(j) => {
                out.push_str(self.raw(j.opening));
                for child in &j.children {
                    self.inline(child, out);
                }
                if let Some(closing) = j.closing {
                    out.push_str(self.raw(closing));
                }
            }
        }
    }

    fn link_kind(&self, kind: &LinkKind<'_>, out: &mut String) {
        match kind {
            LinkKind::Inline { destination, title } => {
                // `<dest>` vs bare is a style fact; the destination text is not.
                let dest = decode::destination(self.source, destination);
                let _ = write!(out, "href={dest}");
                if let Some(title) = title {
                    let _ = write!(out, " title={:?}", decode::title(self.source, title));
                }
            }
            LinkKind::Reference { kind, label: pieces } => {
                let kind = match kind {
                    ReferenceKind::Full => "full",
                    ReferenceKind::Collapsed => "collapsed",
                    ReferenceKind::Shortcut => "shortcut",
                };
                let label = label::normalize(&self.joined(pieces)).into_owned();
                let _ = write!(out, "ref={kind} [{label}]");
            }
        }
    }
}

/// Whitespace runs are a formatter freedom (`proseWrap`, indentation), their content is not.
fn collapse_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
