// Port of internal/rule/rule.go: rule messages, fixes, diagnostics and the per-file rule context.
//
// Go rules are `Run(ctx, options) RuleListeners`: a closure per file that returns listeners keyed by
// node kind. Here a rule is created once per distinct options value (`Rule`), and per file it creates a
// `RuleVisitor` that holds the file-local state the Go closure would capture.

use tsrs_ast::{Kind, Node, SourceFile};
use tsrs_checker::Checker;
use tsrs_compiler::Program;
use tsrs_core::P;

use crate::protocol;
use crate::utils::trim_node_text_range;

/// Listener keys. Go encodes these as `kind + 1000 * n` (rule.ListenerOnExit & co.).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Listener {
    Enter(Kind),
    Exit(Kind),
    AllowPattern(Kind),
    AllowPatternExit(Kind),
    NotAllowPattern(Kind),
    NotAllowPatternExit(Kind),
}

pub const KIND_SLOTS: usize = 1000;

impl Listener {
    #[inline]
    pub fn slot(self) -> usize {
        let (base, kind) = match self {
            Listener::Enter(k) => (0, k),
            Listener::Exit(k) => (1, k),
            Listener::AllowPattern(k) => (2, k),
            Listener::AllowPatternExit(k) => (3, k),
            Listener::NotAllowPattern(k) => (4, k),
            Listener::NotAllowPatternExit(k) => (5, k),
        };
        base * KIND_SLOTS + kind as i16 as usize
    }
}

#[derive(Clone, Debug)]
pub struct RuleMessage {
    pub id: &'static str,
    pub description: String,
    pub help: Option<String>,
}

impl RuleMessage {
    pub fn new(id: &'static str, description: impl Into<String>) -> RuleMessage {
        RuleMessage { id, description: description.into(), help: None }
    }
    pub fn with_help(
        id: &'static str,
        description: impl Into<String>,
        help: impl Into<String>,
    ) -> RuleMessage {
        RuleMessage { id, description: description.into(), help: Some(help.into()) }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleFix {
    pub text: String,
    pub pos: i32,
    pub end: i32,
}

#[derive(Clone, Debug)]
pub struct RuleSuggestion {
    pub message: RuleMessage,
    pub fixes: Vec<RuleFix>,
}

#[derive(Clone, Debug)]
pub struct LabeledRange {
    pub label: String,
    pub pos: i32,
    pub end: i32,
}

/// A diagnostic as a rule builds it (Go rule.RuleDiagnostic without RuleName/SourceFile, which the
/// context fills in).
#[derive(Clone, Debug)]
pub struct RuleDiagnostic {
    pub pos: i32,
    pub end: i32,
    pub message: RuleMessage,
    pub labeled_ranges: Vec<LabeledRange>,
}

/// A reported diagnostic with everything the output needs.
#[derive(Clone, Debug)]
pub struct ReportedDiagnostic {
    pub rule_name: &'static str,
    pub file: P<SourceFile>,
    pub pos: i32,
    pub end: i32,
    pub message: RuleMessage,
    pub labeled_ranges: Vec<LabeledRange>,
    /// None when no fixes were provided (or fixes were not requested).
    pub fixes: Option<Vec<RuleFix>>,
    pub suggestions: Option<Vec<RuleSuggestion>>,
}

impl ReportedDiagnostic {
    pub fn to_protocol(&self) -> protocol::Diagnostic {
        let msg = |m: &RuleMessage| protocol::Message {
            id: m.id.to_string(),
            description: m.description.clone(),
            help: m.help.clone(),
        };
        let fixes = |fs: &[RuleFix]| -> Vec<protocol::Fix> {
            fs.iter()
                .map(|f| protocol::Fix {
                    text: f.text.clone(),
                    range: protocol::Range { pos: f.pos, end: f.end },
                })
                .collect()
        };
        protocol::Diagnostic {
            kind: 0,
            range: Some(protocol::Range { pos: self.pos, end: self.end }),
            message: msg(&self.message),
            file_path: Some(self.file.file_name().to_string()),
            labeled_ranges: self
                .labeled_ranges
                .iter()
                .map(|l| protocol::LabeledRange {
                    label: l.label.clone(),
                    range: protocol::Range { pos: l.pos, end: l.end },
                })
                .collect(),
            rule: Some(self.rule_name.to_string()),
            fixes: self.fixes.as_deref().map(fixes).unwrap_or_default(),
            suggestions: self
                .suggestions
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|s| protocol::Suggestion { message: msg(&s.message), fixes: fixes(&s.fixes) })
                .collect(),
        }
    }
}

/// Go rule.RuleContext. One per (file, rule) invocation; `checker` is the worker's checker.
pub struct Ctx<'a> {
    pub checker: &'a mut Checker,
    pub program: &'static Program,
    pub file: P<SourceFile>,
    pub fix: bool,
    pub fix_suggestions: bool,
    pub rule_name: &'static str,
    pub out: &'a mut Vec<ReportedDiagnostic>,
}

impl<'a> Ctx<'a> {
    #[inline]
    pub fn text(&self) -> &'static str {
        self.file.text()
    }

    #[inline]
    pub fn trim(&self, node: P<Node>) -> (i32, i32) {
        trim_node_text_range(self.file, node)
    }

    fn emit(
        &mut self,
        d: RuleDiagnostic,
        fixes: Option<Vec<RuleFix>>,
        suggestions: Option<Vec<RuleSuggestion>>,
    ) {
        self.out.push(ReportedDiagnostic {
            rule_name: self.rule_name,
            file: self.file,
            pos: d.pos,
            end: d.end,
            message: d.message,
            labeled_ranges: d.labeled_ranges,
            fixes,
            suggestions,
        });
    }

    pub fn report_diagnostic(&mut self, d: RuleDiagnostic) {
        self.emit(d, None, None);
    }

    pub fn report_diagnostic_with_fixes(
        &mut self,
        d: RuleDiagnostic,
        fixes: impl FnOnce(&mut Self) -> Vec<RuleFix>,
    ) {
        let f = if self.fix { fixes(self) } else { Vec::new() };
        self.emit(d, Some(f), None);
    }

    pub fn report_diagnostic_with_suggestions(
        &mut self,
        d: RuleDiagnostic,
        suggestions: impl FnOnce(&mut Self) -> Vec<RuleSuggestion>,
    ) {
        let s = if self.fix_suggestions { suggestions(self) } else { Vec::new() };
        self.emit(d, None, Some(s));
    }

    pub fn report_range(&mut self, pos: i32, end: i32, message: RuleMessage) {
        self.emit(RuleDiagnostic { pos, end, message, labeled_ranges: Vec::new() }, None, None);
    }

    pub fn report_node(&mut self, node: P<Node>, message: RuleMessage) {
        let (pos, end) = self.trim(node);
        self.report_range(pos, end, message);
    }

    pub fn report_node_with_fixes(
        &mut self,
        node: P<Node>,
        message: RuleMessage,
        fixes: impl FnOnce(&mut Self) -> Vec<RuleFix>,
    ) {
        let (pos, end) = self.trim(node);
        self.report_diagnostic_with_fixes(
            RuleDiagnostic { pos, end, message, labeled_ranges: Vec::new() },
            fixes,
        );
    }

    pub fn report_node_with_suggestions(
        &mut self,
        node: P<Node>,
        message: RuleMessage,
        suggestions: impl FnOnce(&mut Self) -> Vec<RuleSuggestion>,
    ) {
        let (pos, end) = self.trim(node);
        self.report_diagnostic_with_suggestions(
            RuleDiagnostic { pos, end, message, labeled_ranges: Vec::new() },
            suggestions,
        );
    }

    /// Go rule.ReportNodeWithFixesOrSuggestions.
    pub fn report_node_with_fixes_or_suggestions(
        &mut self,
        node: P<Node>,
        fix: bool,
        message: RuleMessage,
        suggestion_message: RuleMessage,
        fixes: Vec<RuleFix>,
    ) {
        if fix {
            self.report_node_with_fixes(node, message, |_| fixes);
        } else {
            self.report_node_with_suggestions(node, message, |_| {
                vec![RuleSuggestion { message: suggestion_message, fixes }]
            });
        }
    }

    // ---- fix builders (rule.RuleFix*) ----

    pub fn fix_insert_before(&self, node: P<Node>, text: impl Into<String>) -> RuleFix {
        let (pos, _) = self.trim(node);
        RuleFix { text: text.into(), pos, end: pos }
    }
    pub fn fix_insert_after(&self, node: P<Node>, text: impl Into<String>) -> RuleFix {
        RuleFix { text: text.into(), pos: node.end(), end: node.end() }
    }
    pub fn fix_replace(&self, node: P<Node>, text: impl Into<String>) -> RuleFix {
        let (pos, end) = self.trim(node);
        RuleFix { text: text.into(), pos, end }
    }
    pub fn fix_replace_range(&self, pos: i32, end: i32, text: impl Into<String>) -> RuleFix {
        RuleFix { text: text.into(), pos, end }
    }
    pub fn fix_remove_range(&self, pos: i32, end: i32) -> RuleFix {
        RuleFix { text: String::new(), pos, end }
    }
}

/// Per-file state of a rule (the Go closure returned by `Run`).
pub trait RuleVisitor {
    fn listeners(&self) -> &[Listener];
    fn visit(&mut self, ctx: &mut Ctx, listener: Listener, node: P<Node>);
}

/// A configured rule (rule + parsed options). Shared by all workers.
pub trait Rule: Send + Sync {
    fn name(&self) -> &'static str;
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor>;
}

/// Error-free option parsing helper: Go's UnmarshalOptions marshals `options` (nil -> `null`) and
/// unmarshals it into the options struct, whose UnmarshalJSON fills defaults.
pub fn options_object(
    options: Option<&serde_json::Value>,
) -> serde_json::Map<String, serde_json::Value> {
    match options {
        Some(serde_json::Value::Object(m)) => m.clone(),
        // tsgolint test helpers sometimes pass a one-element array (ESLint style).
        Some(serde_json::Value::Array(a)) if a.len() == 1 => match &a[0] {
            serde_json::Value::Object(m) => m.clone(),
            _ => Default::default(),
        },
        _ => Default::default(),
    }
}

pub fn opt_bool(m: &serde_json::Map<String, serde_json::Value>, key: &str, default: bool) -> bool {
    match m.get(key) {
        Some(serde_json::Value::Bool(b)) => *b,
        _ => default,
    }
}
