pub const GLOBAL_OBJECT_NAMES: [&str; 4] = ["global", "globalThis", "self", "window"];

/// set of reserved HTML tag names definition
/// if it's not reserved, then it can have aria-* roles, states, and properties
/// Reference: <https://github.com/GoogleChrome/accessibility-developer-tools/wiki/Audit-Rules#ax_aria_12>
/// Reference: <https://www.w3.org/TR/html-aria/#rules-wd>
/// Reference: <https://github.com/A11yance/aria-query/blob/v5.3.2/src/domMap.js>
pub const RESERVED_HTML_TAG: [&str; 16] = [
    "base", "col", "colgroup", "head", "html", "link", "meta", "noembed", "noscript", "param",
    "picture", "script", "source", "style", "title", "track",
];

const ARIA_ACTIVEDESCENDANT: &str = "activedescendant";
const ARIA_ATOMIC: &str = "atomic";
const ARIA_AUTOCOMPLETE: &str = "autocomplete";
const ARIA_BUSY: &str = "busy";
const ARIA_BRAILLELABEL: &str = "braillelabel";
const ARIA_BRAILLEROLEDESCRIPTION: &str = "brailleroledescription";
const ARIA_CHECKED: &str = "checked";
const ARIA_COLCOUNT: &str = "colcount";
const ARIA_COLINDEX: &str = "colindex";
const ARIA_COLSPAN: &str = "colspan";
const ARIA_CONTROLS: &str = "controls";
const ARIA_CURRENT: &str = "current";
const ARIA_DESCRIBEDBY: &str = "describedby";
const ARIA_DESCRIPTION: &str = "description";
const ARIA_DETAILS: &str = "details";
const ARIA_DISABLED: &str = "disabled";
const ARIA_DROPEFFECT: &str = "dropeffect";
const ARIA_ERRORMESSAGE: &str = "errormessage";
const ARIA_EXPANDED: &str = "expanded";
const ARIA_FLOWTO: &str = "flowto";
const ARIA_GRABBED: &str = "grabbed";
const ARIA_HASPOPUP: &str = "haspopup";
const ARIA_HIDDEN: &str = "hidden";
const ARIA_INVALID: &str = "invalid";
const ARIA_KEYSHORTCUTS: &str = "keyshortcuts";
const ARIA_LABEL: &str = "label";
const ARIA_LABELLEDBY: &str = "labelledby";
const ARIA_LEVEL: &str = "level";
const ARIA_LIVE: &str = "live";
const ARIA_MODAL: &str = "modal";
const ARIA_MULTILINE: &str = "multiline";
const ARIA_MULTISELECTABLE: &str = "multiselectable";
const ARIA_ORIENTATION: &str = "orientation";
const ARIA_OWNS: &str = "owns";
const ARIA_PLACEHOLDER: &str = "placeholder";
const ARIA_POSINSET: &str = "posinset";
const ARIA_PRESSED: &str = "pressed";
const ARIA_READONLY: &str = "readonly";
const ARIA_RELEVANT: &str = "relevant";
const ARIA_REQUIRED: &str = "required";
const ARIA_ROLEDESCRIPTION: &str = "roledescription";
const ARIA_ROWCOUNT: &str = "rowcount";
const ARIA_ROWINDEX: &str = "rowindex";
const ARIA_ROWSPAN: &str = "rowspan";
const ARIA_SELECTED: &str = "selected";
const ARIA_SETSIZE: &str = "setsize";
const ARIA_SORT: &str = "sort";
const ARIA_VALUEMAX: &str = "valuemax";
const ARIA_VALUEMIN: &str = "valuemin";
const ARIA_VALUENOW: &str = "valuenow";
const ARIA_VALUETEXT: &str = "valuetext";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AriaProperty {
    ActiveDescendant,
    Atomic,
    AutoComplete,
    Busy,
    BrailleLabel,
    BrailleRoleDescription,
    Checked,
    ColCount,
    ColIndex,
    ColSpan,
    Controls,
    Current,
    DescribedBy,
    Description,
    Details,
    Disabled,
    DropEffect,
    ErrorMessage,
    Expanded,
    FlowTo,
    Grabbed,
    HasPopup,
    Hidden,
    Invalid,
    KeyShortcuts,
    Label,
    LabelledBy,
    Level,
    Live,
    Modal,
    Multiline,
    Multiselectable,
    Orientation,
    Owns,
    Placeholder,
    PosInSet,
    Pressed,
    Readonly,
    Relevant,
    Required,
    RoleDescription,
    RowCount,
    RowIndex,
    RowSpan,
    Selected,
    SetSize,
    Sort,
    ValueMax,
    ValueMin,
    ValueNow,
    ValueText,
}

impl TryFrom<&str> for AriaProperty {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() < 9 || &value.as_bytes()[..5] != b"aria-" {
            return Err(());
        }
        match &value[5..] {
            ARIA_ACTIVEDESCENDANT => Ok(AriaProperty::ActiveDescendant),
            ARIA_ATOMIC => Ok(AriaProperty::Atomic),
            ARIA_AUTOCOMPLETE => Ok(AriaProperty::AutoComplete),
            ARIA_BUSY => Ok(AriaProperty::Busy),
            ARIA_BRAILLELABEL => Ok(AriaProperty::BrailleLabel),
            ARIA_BRAILLEROLEDESCRIPTION => Ok(AriaProperty::BrailleRoleDescription),
            ARIA_CHECKED => Ok(AriaProperty::Checked),
            ARIA_COLCOUNT => Ok(AriaProperty::ColCount),
            ARIA_COLINDEX => Ok(AriaProperty::ColIndex),
            ARIA_COLSPAN => Ok(AriaProperty::ColSpan),
            ARIA_CONTROLS => Ok(AriaProperty::Controls),
            ARIA_CURRENT => Ok(AriaProperty::Current),
            ARIA_DESCRIBEDBY => Ok(AriaProperty::DescribedBy),
            ARIA_DESCRIPTION => Ok(AriaProperty::Description),
            ARIA_DETAILS => Ok(AriaProperty::Details),
            ARIA_DISABLED => Ok(AriaProperty::Disabled),
            ARIA_DROPEFFECT => Ok(AriaProperty::DropEffect),
            ARIA_ERRORMESSAGE => Ok(AriaProperty::ErrorMessage),
            ARIA_EXPANDED => Ok(AriaProperty::Expanded),
            ARIA_FLOWTO => Ok(AriaProperty::FlowTo),
            ARIA_GRABBED => Ok(AriaProperty::Grabbed),
            ARIA_HASPOPUP => Ok(AriaProperty::HasPopup),
            ARIA_HIDDEN => Ok(AriaProperty::Hidden),
            ARIA_INVALID => Ok(AriaProperty::Invalid),
            ARIA_KEYSHORTCUTS => Ok(AriaProperty::KeyShortcuts),
            ARIA_LABEL => Ok(AriaProperty::Label),
            ARIA_LABELLEDBY => Ok(AriaProperty::LabelledBy),
            ARIA_LEVEL => Ok(AriaProperty::Level),
            ARIA_LIVE => Ok(AriaProperty::Live),
            ARIA_MODAL => Ok(AriaProperty::Modal),
            ARIA_MULTILINE => Ok(AriaProperty::Multiline),
            ARIA_MULTISELECTABLE => Ok(AriaProperty::Multiselectable),
            ARIA_ORIENTATION => Ok(AriaProperty::Orientation),
            ARIA_OWNS => Ok(AriaProperty::Owns),
            ARIA_PLACEHOLDER => Ok(AriaProperty::Placeholder),
            ARIA_POSINSET => Ok(AriaProperty::PosInSet),
            ARIA_PRESSED => Ok(AriaProperty::Pressed),
            ARIA_READONLY => Ok(AriaProperty::Readonly),
            ARIA_RELEVANT => Ok(AriaProperty::Relevant),
            ARIA_REQUIRED => Ok(AriaProperty::Required),
            ARIA_ROLEDESCRIPTION => Ok(AriaProperty::RoleDescription),
            ARIA_ROWCOUNT => Ok(AriaProperty::RowCount),
            ARIA_ROWINDEX => Ok(AriaProperty::RowIndex),
            ARIA_ROWSPAN => Ok(AriaProperty::RowSpan),
            ARIA_SELECTED => Ok(AriaProperty::Selected),
            ARIA_SETSIZE => Ok(AriaProperty::SetSize),
            ARIA_SORT => Ok(AriaProperty::Sort),
            ARIA_VALUEMAX => Ok(AriaProperty::ValueMax),
            ARIA_VALUEMIN => Ok(AriaProperty::ValueMin),
            ARIA_VALUENOW => Ok(AriaProperty::ValueNow),
            ARIA_VALUETEXT => Ok(AriaProperty::ValueText),
            _ => Err(()),
        }
    }
}

/// Returns whether this string is a valid ARIA property.
///
/// # Example
/// - `is_valid_aria_property("aria-label")` => `true`
/// - `is_valid_aria_property("aria-labelby")` => `false`
pub fn is_valid_aria_property(name: &str) -> bool {
    AriaProperty::try_from(name).is_ok()
}

/// set of valid ARIA role definitions
/// Reference: <https://www.w3.org/TR/wai-aria/#role_definitions>
/// Reference: <https://github.com/A11yance/aria-query/blob/v5.3.2/src/rolesMap.js>
pub const VALID_ARIA_ROLES: phf::Set<&'static str> = phf::phf_set![
    "alert",
    "alertdialog",
    "application",
    "article",
    "banner",
    "blockquote",
    "button",
    "caption",
    "cell",
    "checkbox",
    "code",
    "columnheader",
    "combobox",
    "complementary",
    "contentinfo",
    "definition",
    "deletion",
    "dialog",
    "directory",
    "doc-abstract",
    "doc-acknowledgments",
    "doc-afterword",
    "doc-appendix",
    "doc-backlink",
    "doc-biblioentry",
    "doc-bibliography",
    "doc-biblioref",
    "doc-chapter",
    "doc-colophon",
    "doc-conclusion",
    "doc-cover",
    "doc-credit",
    "doc-credits",
    "doc-dedication",
    "doc-endnote",
    "doc-endnotes",
    "doc-epigraph",
    "doc-epilogue",
    "doc-errata",
    "doc-example",
    "doc-footnote",
    "doc-foreword",
    "doc-glossary",
    "doc-glossref",
    "doc-index",
    "doc-introduction",
    "doc-noteref",
    "doc-notice",
    "doc-pagebreak",
    "doc-pagelist",
    "doc-part",
    "doc-preface",
    "doc-prologue",
    "doc-pullquote",
    "doc-qna",
    "doc-subtitle",
    "doc-tip",
    "doc-toc",
    "document",
    "emphasis",
    "feed",
    "figure",
    "form",
    "generic",
    "graphics-document",
    "graphics-object",
    "graphics-symbol",
    "grid",
    "gridcell",
    "group",
    "heading",
    "img",
    "insertion",
    "link",
    "list",
    "listbox",
    "listitem",
    "log",
    "main",
    "mark",
    "marquee",
    "math",
    "menu",
    "menubar",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "meter",
    "navigation",
    "none",
    "note",
    "option",
    "paragraph",
    "presentation",
    "progressbar",
    "radio",
    "radiogroup",
    "region",
    "row",
    "rowgroup",
    "rowheader",
    "scrollbar",
    "search",
    "searchbox",
    "separator",
    "slider",
    "spinbutton",
    "status",
    "strong",
    "subscript",
    "superscript",
    "switch",
    "tab",
    "table",
    "tablist",
    "tabpanel",
    "term",
    "textbox",
    "time",
    "timer",
    "toolbar",
    "tooltip",
    "tree",
    "treegrid",
    "treeitem",
];

pub const HTML_TAG: phf::Set<&'static str> = phf::phf_set![
    "a",
    "abbr",
    "acronym",
    "address",
    "applet",
    "area",
    "article",
    "aside",
    "audio",
    "b",
    "base",
    "basefont",
    "bdi",
    "bdo",
    "bgsound",
    "big",
    "blink",
    "blockquote",
    "body",
    "br",
    "button",
    "canvas",
    "caption",
    "center",
    "cite",
    "code",
    "col",
    "colgroup",
    "command",
    "content",
    "data",
    "datalist",
    "dd",
    "del",
    "details",
    "dfn",
    "dialog",
    "dir",
    "div",
    "dl",
    "dt",
    "element",
    "em",
    "embed",
    "fieldset",
    "figcaption",
    "figure",
    "font",
    "footer",
    "form",
    "frame",
    "frameset",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hgroup",
    "hr",
    "html",
    "i",
    "iframe",
    "image",
    "img",
    "input",
    "ins",
    "isindex",
    "kbd",
    "keygen",
    "label",
    "legend",
    "li",
    "link",
    "listing",
    "main",
    "map",
    "mark",
    "marquee",
    "math",
    "menu",
    "menuitem",
    "meta",
    "meter",
    "multicol",
    "nav",
    "nextid",
    "nobr",
    "noembed",
    "noframes",
    "noscript",
    "object",
    "ol",
    "optgroup",
    "option",
    "output",
    "p",
    "param",
    "picture",
    "plaintext",
    "pre",
    "progress",
    "q",
    "rb",
    "rbc",
    "rp",
    "rt",
    "rtc",
    "ruby",
    "s",
    "samp",
    "script",
    "search",
    "section",
    "select",
    "shadow",
    "slot",
    "small",
    "source",
    "spacer",
    "span",
    "strike",
    "strong",
    "style",
    "sub",
    "summary",
    "sup",
    "svg",
    "table",
    "tbody",
    "td",
    "template",
    "textarea",
    "tfoot",
    "th",
    "thead",
    "time",
    "title",
    "tr",
    "track",
    "tt",
    "u",
    "ul",
    "var",
    "video",
    "wbr",
    "xmp",
];
