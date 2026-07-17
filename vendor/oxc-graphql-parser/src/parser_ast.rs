use crate::ast::*;
use crate::lexer::Lexer;
use crate::{Error, LimitTracker, T, Token, TokenKind};
use oxc_allocator::{Allocator, Box as ArenaBox, Vec as ArenaVec};
use std::ops::ControlFlow;

pub struct Parser<'a> {
    allocator: &'a Allocator,
    input: &'a str,
    lexer: Lexer<'a>,
    current_token: Option<Token<'a>>,
    errors: Vec<Error>,
    comments: Vec<Span>,
    recursion_limit: LimitTracker,
    accept_errors: bool,
    experimental_fragment_arguments: bool,
    last_end: u32,
    /// Reusable scratch stack for building AST lists. List elements are
    /// collected here and copied into the arena once, at the exact final
    /// size: growing an arena vec strands every outgrown copy in the arena,
    /// because parsing the elements allocates in between. Recursive descent
    /// finishes nested lists in LIFO order, so all list types share this one
    /// stack: a list records the stack length when it starts and drains
    /// everything above that mark when it ends.
    scratch: Vec<ScratchNode<'a>>,
}

/// An element of [`Parser::scratch`]; one variant per AST list type built
/// through the scratch stack.
enum ScratchNode<'a> {
    Definition(Definition<'a>),
    Selection(Selection<'a>),
    Argument(Argument<'a>),
    VariableDefinition(VariableDefinition<'a>),
    FieldDefinition(FieldDefinition<'a>),
    InputValueDefinition(InputValueDefinition<'a>),
    Directive(Directive<'a>),
}

#[derive(Clone, Copy)]
enum Constness {
    Const,
    NotConst,
}

const DEFAULT_RECURSION_LIMIT: usize = 500;

impl<'a> Parser<'a> {
    /// # Panics
    ///
    /// Panics if `input` is larger than 4 GiB: AST spans store `u32` offsets.
    pub fn new(allocator: &'a Allocator, input: &'a str) -> Self {
        assert!(
            u32::try_from(input.len()).is_ok(),
            "source text is too long for u32 spans (max 4 GiB): {} bytes",
            input.len()
        );
        Self {
            allocator,
            input,
            lexer: Lexer::new(input),
            current_token: None,
            errors: Vec::new(),
            comments: Vec::new(),
            recursion_limit: LimitTracker::new(DEFAULT_RECURSION_LIMIT),
            accept_errors: true,
            experimental_fragment_arguments: false,
            last_end: 0,
            scratch: Vec::new(),
        }
    }

    pub fn recursion_limit(mut self, recursion_limit: usize) -> Self {
        self.recursion_limit = LimitTracker::new(recursion_limit);
        self
    }

    pub fn token_limit(mut self, token_limit: usize) -> Self {
        self.lexer = self.lexer.with_limit(token_limit);
        self
    }

    pub fn experimental_fragment_arguments(mut self, allow: bool) -> Self {
        self.experimental_fragment_arguments = allow;
        self
    }

    pub fn parse(mut self) -> Ast<'a, Document<'a>> {
        let document = self.parse_document();
        self.into_ast(document)
    }

    pub fn parse_selection_set(mut self) -> Ast<'a, SelectionSet<'a>> {
        let selection_set = self.parse_selection_set_inner();
        self.into_ast(selection_set)
    }

    pub fn parse_type(mut self) -> Ast<'a, Type<'a>> {
        let ty = self.parse_type_inner().unwrap_or_else(|| {
            let span = self.current_span();
            self.err("expected a type");
            Type::Missing(span)
        });
        self.into_ast(ty)
    }

    fn into_ast<T>(self, root: T) -> Ast<'a, T> {
        let token_limit = self.lexer.limit_tracker;
        Ast::new(self.input, root, self.errors, self.comments, self.recursion_limit, token_limit)
    }

    /// Marks the start of a new scratch-built list, pre-sizing the stack so
    /// small parses pay for at most one scratch allocation.
    fn scratch_mark(&mut self) -> usize {
        if self.scratch.capacity() == 0 {
            self.scratch.reserve(128);
        }
        self.scratch.len()
    }

    /// Moves the scratch elements above `mark` into an exact-size arena vec.
    #[inline]
    fn drain_scratch<T>(
        &mut self,
        mark: usize,
        unwrap: impl FnMut(ScratchNode<'a>) -> T,
    ) -> ArenaVec<'a, T> {
        ArenaVec::from_iter_in(self.scratch.drain(mark..).map(unwrap), &self.allocator)
    }

    fn parse_document(&mut self) -> Document<'a> {
        let start = self.current_start();
        let mark = self.scratch_mark();

        if self.peek().is_none_or(|kind| kind == TokenKind::Eof) {
            self.err("Unexpected <EOF>.");
        }

        self.peek_while(|parser, kind| {
            if kind == TokenKind::Eof {
                return ControlFlow::Break(());
            }

            let before = parser.current_span();
            if let Some(definition) = parser.parse_definition() {
                parser.scratch.push(ScratchNode::Definition(definition));
            } else {
                parser.err_and_pop("expected a StringValue, Name or OperationDefinition");
            }

            if parser.current_span() == before && parser.peek() != Some(TokenKind::Eof) {
                parser.bump();
            }

            ControlFlow::Continue(())
        });

        let definitions = self.drain_scratch(mark, |node| match node {
            ScratchNode::Definition(definition) => definition,
            _ => unreachable!("scratch stack discipline"),
        });
        Document { definitions, span: self.span_from(start) }
    }

    fn parse_definition(&mut self) -> Option<Definition<'a>> {
        let description = self.parse_description_if_present();
        let selector = self.peek_data()?;

        let definition = match selector {
            "directive" => {
                let definition = self.parse_directive_definition(description);
                Definition::Directive(ArenaBox::new_in(definition, &self.allocator))
            }
            "enum" => {
                let definition = self.parse_enum_type_definition(description);
                Definition::EnumType(ArenaBox::new_in(definition, &self.allocator))
            }
            "extend" => {
                if description.is_some() {
                    self.err(
                        "Unexpected description, only GraphQL definitions support descriptions.",
                    );
                }
                return self.parse_extension();
            }
            "fragment" => {
                let definition = self.parse_fragment_definition(description);
                Definition::Fragment(ArenaBox::new_in(definition, &self.allocator))
            }
            "input" => {
                let definition = self.parse_input_object_type_definition(description);
                Definition::InputObjectType(ArenaBox::new_in(definition, &self.allocator))
            }
            "interface" => {
                let definition = self.parse_interface_type_definition(description);
                Definition::InterfaceType(ArenaBox::new_in(definition, &self.allocator))
            }
            "type" => {
                let definition = self.parse_object_type_definition(description);
                Definition::ObjectType(ArenaBox::new_in(definition, &self.allocator))
            }
            "{" => {
                if description.is_some() {
                    self.err(
                        "Unexpected description, descriptions are not supported on shorthand queries.",
                    );
                }
                let definition = self.parse_operation_definition(description);
                Definition::Operation(ArenaBox::new_in(definition, &self.allocator))
            }
            "query" | "mutation" | "subscription" => {
                let definition = self.parse_operation_definition(description);
                Definition::Operation(ArenaBox::new_in(definition, &self.allocator))
            }
            "scalar" => {
                let definition = self.parse_scalar_type_definition(description);
                Definition::ScalarType(ArenaBox::new_in(definition, &self.allocator))
            }
            "schema" => {
                let definition = self.parse_schema_definition(description);
                Definition::Schema(ArenaBox::new_in(definition, &self.allocator))
            }
            "union" => {
                let definition = self.parse_union_type_definition(description);
                Definition::UnionType(ArenaBox::new_in(definition, &self.allocator))
            }
            _ => {
                if description.is_some() {
                    self.err("expected a definition after this StringValue");
                } else {
                    self.err_and_pop("expected definition");
                }
                return None;
            }
        };

        Some(definition)
    }

    fn parse_extension(&mut self) -> Option<Definition<'a>> {
        let start = self.current_start();
        self.expect_name_value("extend");

        let definition = match self.peek_data() {
            Some("schema") => {
                let extension = self.parse_schema_extension_from(start);
                Definition::SchemaExtension(ArenaBox::new_in(extension, &self.allocator))
            }
            Some("scalar") => {
                let extension = self.parse_scalar_type_extension_from(start);
                Definition::ScalarTypeExtension(ArenaBox::new_in(extension, &self.allocator))
            }
            Some("type") => {
                let extension = self.parse_object_type_extension_from(start);
                Definition::ObjectTypeExtension(ArenaBox::new_in(extension, &self.allocator))
            }
            Some("interface") => {
                let extension = self.parse_interface_type_extension_from(start);
                Definition::InterfaceTypeExtension(ArenaBox::new_in(extension, &self.allocator))
            }
            Some("union") => {
                let extension = self.parse_union_type_extension_from(start);
                Definition::UnionTypeExtension(ArenaBox::new_in(extension, &self.allocator))
            }
            Some("enum") => {
                let extension = self.parse_enum_type_extension_from(start);
                Definition::EnumTypeExtension(ArenaBox::new_in(extension, &self.allocator))
            }
            Some("input") => {
                let extension = self.parse_input_object_type_extension_from(start);
                Definition::InputObjectTypeExtension(ArenaBox::new_in(extension, &self.allocator))
            }
            Some("directive") => {
                let extension = self.parse_directive_extension_from(start);
                Definition::DirectiveExtension(ArenaBox::new_in(extension, &self.allocator))
            }
            _ => {
                self.err("expected a valid extension");
                return None;
            }
        };

        Some(definition)
    }

    fn parse_operation_definition(
        &mut self,
        description: Option<ArenaBox<'a, StringValue<'a>>>,
    ) -> OperationDefinition<'a> {
        let start = self.definition_start(&description);

        if self.peek() == Some(T!['{']) {
            let selection_set = Some(self.parse_alloc_selection_set());
            return OperationDefinition {
                description,
                operation_type: OperationType::Query,
                name: None,
                variable_definitions: ArenaVec::new_in(&self.allocator),
                directives: ArenaVec::new_in(&self.allocator),
                selection_set,
                span: self.span_from(start),
            };
        }

        let operation_type = self.parse_operation_type("expected Operation Type");

        let name = if self.peek() == Some(TokenKind::Name) { self.parse_name() } else { None };
        let variable_definitions = self.parse_variable_definitions_if_present();
        let directives = self.parse_directives(Constness::NotConst);
        let selection_set = self.parse_required_selection_set();

        OperationDefinition {
            description,
            operation_type,
            name,
            variable_definitions,
            directives,
            selection_set,
            span: self.span_from(start),
        }
    }

    fn parse_fragment_definition(
        &mut self,
        description: Option<ArenaBox<'a, StringValue<'a>>>,
    ) -> FragmentDefinition<'a> {
        let start = self.definition_start(&description);
        self.expect_name_value("fragment");
        let name = self.parse_name().unwrap_or_else(|| self.missing_name());

        let variable_definitions = if self.experimental_fragment_arguments {
            self.parse_variable_definitions_if_present()
        } else {
            ArenaVec::new_in(&self.allocator)
        };

        self.expect_name_value("on");
        let type_condition = self.parse_named_type().unwrap_or_else(|| self.missing_named_type());
        let directives = self.parse_directives(Constness::NotConst);
        let selection_set = self.parse_required_selection_set();

        FragmentDefinition {
            description,
            name,
            variable_definitions,
            type_condition,
            directives,
            selection_set,
            span: self.span_from(start),
        }
    }

    fn parse_alloc_selection_set(&mut self) -> ArenaBox<'a, SelectionSet<'a>> {
        let selection_set = self.parse_selection_set_inner();
        ArenaBox::new_in(selection_set, &self.allocator)
    }

    /// Parse a selection set that is required in this position. If the next
    /// token is not `{`, records an error and returns `None`.
    fn parse_required_selection_set(&mut self) -> Option<ArenaBox<'a, SelectionSet<'a>>> {
        if self.peek() == Some(T!['{']) {
            Some(self.parse_alloc_selection_set())
        } else {
            self.err("expected a Selection Set");
            None
        }
    }

    /// Parse an operation type keyword (`query`, `mutation`, or `subscription`),
    /// consuming it. If none is present, records an error using `missing` and
    /// falls back to `OperationType::Query`.
    fn parse_operation_type(&mut self, missing: &str) -> OperationType {
        match self.peek_data() {
            Some("query") => {
                self.bump();
                OperationType::Query
            }
            Some("mutation") => {
                self.bump();
                OperationType::Mutation
            }
            Some("subscription") => {
                self.bump();
                OperationType::Subscription
            }
            _ => {
                self.err(missing);
                OperationType::Query
            }
        }
    }

    fn parse_selection_set_inner(&mut self) -> SelectionSet<'a> {
        let start = self.current_start();
        self.expect(T!['{'], "expected {");

        let mark = self.scratch_mark();

        self.peek_while(|parser, kind| match kind {
            T!['}'] => {
                if parser.scratch.len() == mark {
                    parser.err("expected Selection");
                }
                parser.bump();
                ControlFlow::Break(())
            }
            TokenKind::Eof => {
                parser.err("expected }");
                ControlFlow::Break(())
            }
            TokenKind::Name | T![...] if parser.recursion_limit.check_and_increment() => {
                parser.limit_err("parser recursion limit reached");
                ControlFlow::Break(())
            }
            TokenKind::Name | T![...] => {
                let selection = parser.parse_selection();
                parser.scratch.push(ScratchNode::Selection(selection));
                parser.recursion_limit.decrement();
                ControlFlow::Continue(())
            }
            _ => {
                parser.err_and_pop("expected a Selection");
                ControlFlow::Continue(())
            }
        });

        let selections = self.drain_scratch(mark, |node| match node {
            ScratchNode::Selection(selection) => selection,
            _ => unreachable!("scratch stack discipline"),
        });

        SelectionSet { selections, span: self.span_from(start) }
    }

    fn parse_selection(&mut self) -> Selection<'a> {
        if self.peek() == Some(T![...]) {
            self.parse_fragment_selection()
        } else {
            let field = self.parse_field();
            Selection::Field(ArenaBox::new_in(field, &self.allocator))
        }
    }

    fn parse_fragment_selection(&mut self) -> Selection<'a> {
        let start = self.current_start();
        self.expect(T![...], "expected ...");

        if self.peek_data() == Some("on") {
            self.bump();
            let type_condition = self.parse_named_type();
            let directives = self.parse_directives(Constness::NotConst);
            let selection_set = self.parse_required_selection_set();
            return Selection::InlineFragment(ArenaBox::new_in(
                InlineFragment {
                    type_condition,
                    directives,
                    selection_set,
                    span: self.span_from(start),
                },
                &self.allocator,
            ));
        }

        if matches!(self.peek(), Some(T![@] | T!['{'])) {
            let directives = self.parse_directives(Constness::NotConst);
            let selection_set = self.parse_required_selection_set();
            return Selection::InlineFragment(ArenaBox::new_in(
                InlineFragment {
                    type_condition: None,
                    directives,
                    selection_set,
                    span: self.span_from(start),
                },
                &self.allocator,
            ));
        }

        let name = self.parse_name().unwrap_or_else(|| self.missing_name());
        let arguments = if self.experimental_fragment_arguments {
            self.parse_arguments_if_present(Constness::NotConst)
        } else {
            ArenaVec::new_in(&self.allocator)
        };
        let directives = self.parse_directives(Constness::NotConst);
        Selection::FragmentSpread(ArenaBox::new_in(
            FragmentSpread { name, arguments, directives, span: self.span_from(start) },
            &self.allocator,
        ))
    }

    fn parse_field(&mut self) -> Field<'a> {
        let start = self.current_start();
        let first_name = self.parse_name().unwrap_or_else(|| self.missing_name());
        let (alias, name) = if self.peek() == Some(T![:]) {
            self.bump();
            let name = self.parse_name().unwrap_or_else(|| self.missing_name());
            (Some(first_name), name)
        } else {
            (None, first_name)
        };

        let arguments = self.parse_arguments_if_present(Constness::NotConst);
        let directives = self.parse_directives(Constness::NotConst);
        let selection_set = if self.peek() == Some(T!['{']) {
            Some(self.parse_alloc_selection_set())
        } else {
            None
        };

        Field { alias, name, arguments, directives, selection_set, span: self.span_from(start) }
    }

    fn parse_arguments_if_present(&mut self, constness: Constness) -> ArenaVec<'a, Argument<'a>> {
        if self.peek() != Some(T!['(']) {
            return ArenaVec::new_in(&self.allocator);
        }

        self.bump();
        let mark = self.scratch_mark();
        self.peek_while(|parser, kind| match kind {
            T![')'] => {
                parser.bump();
                ControlFlow::Break(())
            }
            TokenKind::Name => {
                let argument = parser.parse_argument(constness);
                parser.scratch.push(ScratchNode::Argument(argument));
                ControlFlow::Continue(())
            }
            TokenKind::Eof => {
                parser.err("expected )");
                ControlFlow::Break(())
            }
            _ => {
                parser.err_and_pop("expected an Argument");
                ControlFlow::Continue(())
            }
        });
        self.drain_scratch(mark, |node| match node {
            ScratchNode::Argument(argument) => argument,
            _ => unreachable!("scratch stack discipline"),
        })
    }

    fn parse_argument(&mut self, constness: Constness) -> Argument<'a> {
        let start = self.current_start();
        let name = self.parse_name().unwrap_or_else(|| self.missing_name());
        let value = if self.peek() == Some(T![:]) {
            self.bump();
            Some(self.parse_value(constness, false))
        } else {
            self.err("expected :");
            None
        };
        Argument { name, value, span: self.span_from(start) }
    }

    fn parse_variable_definitions_if_present(&mut self) -> ArenaVec<'a, VariableDefinition<'a>> {
        if self.peek() != Some(T!['(']) {
            return ArenaVec::new_in(&self.allocator);
        }

        self.bump();
        let mark = self.scratch_mark();
        self.peek_while(|parser, kind| match kind {
            T![')'] => {
                if parser.scratch.len() == mark {
                    parser.err("expected a Variable Definition");
                }
                parser.bump();
                ControlFlow::Break(())
            }
            T![$] | TokenKind::StringValue => {
                let definition = parser.parse_variable_definition();
                parser.scratch.push(ScratchNode::VariableDefinition(definition));
                ControlFlow::Continue(())
            }
            TokenKind::Eof => {
                parser.err("expected )");
                ControlFlow::Break(())
            }
            _ => {
                parser.err_and_pop("expected a Variable Definition");
                ControlFlow::Continue(())
            }
        });
        self.drain_scratch(mark, |node| match node {
            ScratchNode::VariableDefinition(definition) => definition,
            _ => unreachable!("scratch stack discipline"),
        })
    }

    fn parse_variable_definition(&mut self) -> VariableDefinition<'a> {
        let start = self.current_start();
        let description = self.parse_description_if_present();
        let variable = self.parse_variable().unwrap_or_else(|| self.missing_variable());
        let mut ty = None;
        let mut default_value = None;
        let mut directives = ArenaVec::new_in(&self.allocator);

        if self.peek() == Some(T![:]) {
            self.bump();
            ty = self.parse_type_inner();
            if self.peek() == Some(T![=]) {
                self.bump();
                default_value = Some(self.parse_value(Constness::Const, false));
            }
            directives = self.parse_directives(Constness::Const);
        } else {
            self.err("expected a Name");
        }

        VariableDefinition {
            description,
            variable,
            ty,
            default_value,
            directives,
            span: self.span_from(start),
        }
    }

    fn parse_variable(&mut self) -> Option<Variable<'a>> {
        let start = self.current_start();
        if self.peek() != Some(T![$]) {
            self.err("expected a Variable");
            return None;
        }
        self.bump();
        let name = self.parse_name().unwrap_or_else(|| self.missing_name());
        Some(Variable { name, span: self.span_from(start) })
    }

    fn parse_directives(&mut self, constness: Constness) -> ArenaVec<'a, Directive<'a>> {
        if self.peek() != Some(T![@]) {
            return ArenaVec::new_in(&self.allocator);
        }

        let mark = self.scratch_mark();
        while self.peek() == Some(T![@]) {
            let directive = self.parse_directive(constness);
            self.scratch.push(ScratchNode::Directive(directive));
        }
        self.drain_scratch(mark, |node| match node {
            ScratchNode::Directive(directive) => directive,
            _ => unreachable!("scratch stack discipline"),
        })
    }

    fn parse_directive(&mut self, constness: Constness) -> Directive<'a> {
        let start = self.current_start();
        self.expect(T![@], "expected @ symbol");
        let name = self.parse_name().unwrap_or_else(|| self.missing_name());
        let arguments = self.parse_arguments_if_present(constness);
        Directive { name, arguments, span: self.span_from(start) }
    }

    fn parse_value(&mut self, constness: Constness, pop_on_error: bool) -> Value<'a> {
        match self.peek() {
            Some(T![$]) => {
                if matches!(constness, Constness::Const) {
                    self.err("unexpected variable value in a Const context");
                }
                match self.parse_variable() {
                    Some(variable) => Value::Variable(ArenaBox::new_in(variable, &self.allocator)),
                    None => Value::Missing(self.current_span()),
                }
            }
            Some(TokenKind::Int) => self.parse_int_value(),
            Some(TokenKind::Float) => self.parse_float_value(),
            Some(TokenKind::StringValue) => match self.parse_string_value() {
                Some(value) => Value::String(ArenaBox::new_in(value, &self.allocator)),
                None => Value::Missing(self.current_span()),
            },
            Some(TokenKind::Name) => self.parse_name_value(),
            Some(T!['[']) => self.parse_list_value(constness),
            Some(T!['{']) => self.parse_object_value(constness),
            _ => {
                let message = "expected a valid Value";
                if pop_on_error {
                    self.err_and_pop(message);
                } else {
                    self.err(message);
                }
                Value::Missing(self.current_span())
            }
        }
    }

    fn parse_int_value(&mut self) -> Value<'a> {
        let token = self.bump().expect("peeked int token must be available");
        Value::Int(ArenaBox::new_in(
            IntValue { raw: token.data(), span: token_span(&token) },
            &self.allocator,
        ))
    }

    fn parse_float_value(&mut self) -> Value<'a> {
        let token = self.bump().expect("peeked float token must be available");
        Value::Float(ArenaBox::new_in(
            FloatValue { raw: token.data(), span: token_span(&token) },
            &self.allocator,
        ))
    }

    fn parse_name_value(&mut self) -> Value<'a> {
        let Some(name) = self.parse_name() else {
            return Value::Missing(self.current_span());
        };
        match name.value {
            "true" => Value::Boolean(ArenaBox::new_in(
                BooleanValue { value: true, span: name.span },
                &self.allocator,
            )),
            "false" => Value::Boolean(ArenaBox::new_in(
                BooleanValue { value: false, span: name.span },
                &self.allocator,
            )),
            "null" => Value::Null(ArenaBox::new_in(NullValue { span: name.span }, &self.allocator)),
            _ => Value::Enum(ArenaBox::new_in(EnumValue { name }, &self.allocator)),
        }
    }

    fn parse_list_value(&mut self, constness: Constness) -> Value<'a> {
        let start = self.current_start();
        self.expect(T!['['], "expected [");
        let mut values = ArenaVec::new_in(&self.allocator);

        self.peek_while(|parser, kind| match kind {
            T![']'] => {
                parser.bump();
                ControlFlow::Break(())
            }
            TokenKind::Eof => {
                parser.err("expected ]");
                ControlFlow::Break(())
            }
            _ if parser.recursion_limit.check_and_increment() => {
                parser.limit_err("parser recursion limit reached");
                ControlFlow::Break(())
            }
            _ => {
                values.push(parser.parse_value(constness, true));
                parser.recursion_limit.decrement();
                ControlFlow::Continue(())
            }
        });

        Value::List(ArenaBox::new_in(
            ListValue { values, span: self.span_from(start) },
            &self.allocator,
        ))
    }

    fn parse_object_value(&mut self, constness: Constness) -> Value<'a> {
        let start = self.current_start();
        self.expect(T!['{'], "expected {");
        let mut fields = ArenaVec::new_in(&self.allocator);

        self.peek_while(|parser, kind| match kind {
            T!['}'] => {
                parser.bump();
                ControlFlow::Break(())
            }
            TokenKind::Name if parser.recursion_limit.check_and_increment() => {
                parser.limit_err("parser recursion limit reached");
                ControlFlow::Break(())
            }
            TokenKind::Name => {
                fields.push(parser.parse_object_field(constness));
                parser.recursion_limit.decrement();
                ControlFlow::Continue(())
            }
            TokenKind::Eof => {
                parser.err("expected }");
                ControlFlow::Break(())
            }
            _ => {
                parser.err_and_pop("expected Object Field");
                ControlFlow::Continue(())
            }
        });

        Value::Object(ArenaBox::new_in(
            ObjectValue { fields, span: self.span_from(start) },
            &self.allocator,
        ))
    }

    fn parse_object_field(&mut self, constness: Constness) -> ObjectField<'a> {
        let start = self.current_start();
        let name = self.parse_name().unwrap_or_else(|| self.missing_name());
        let value = if self.peek() == Some(T![:]) {
            self.bump();
            Some(self.parse_value(constness, true))
        } else {
            self.err("expected :");
            None
        };
        ObjectField { name, value, span: self.span_from(start) }
    }

    fn parse_type_inner(&mut self) -> Option<Type<'a>> {
        let start = self.current_start();
        let mut ty = match self.peek() {
            Some(T!['[']) => {
                self.bump();
                if self.recursion_limit.check_and_increment() {
                    self.limit_err("parser recursion limit reached");
                    return Some(Type::Missing(self.span_from(start)));
                }
                let inner =
                    self.parse_type_inner().unwrap_or_else(|| Type::Missing(self.current_span()));
                self.recursion_limit.decrement();
                self.expect(T![']'], "expected ]");
                Type::List(ArenaBox::new_in(
                    ListType { ty: inner, span: self.span_from(start) },
                    &self.allocator,
                ))
            }
            Some(TokenKind::Name) => {
                let name = self.parse_name().unwrap_or_else(|| self.missing_name());
                Type::Named(ArenaBox::new_in(NamedType { name }, &self.allocator))
            }
            Some(_) => {
                self.err("expected a type");
                return None;
            }
            None => return None,
        };

        if self.peek() == Some(T![!]) {
            self.bump();
            ty = Type::NonNull(ArenaBox::new_in(
                NonNullType { ty, span: self.span_from(start) },
                &self.allocator,
            ));
        }

        Some(ty)
    }

    fn parse_named_type(&mut self) -> Option<NamedType<'a>> {
        self.parse_name().map(|name| NamedType { name })
    }

    fn parse_schema_definition(
        &mut self,
        description: Option<ArenaBox<'a, StringValue<'a>>>,
    ) -> SchemaDefinition<'a> {
        let start = self.definition_start(&description);
        self.expect_name_value("schema");
        let directives = self.parse_directives(Constness::Const);
        let root_operations = self.parse_root_operation_types_if_present();
        SchemaDefinition { description, directives, root_operations, span: self.span_from(start) }
    }

    fn parse_schema_extension_from(&mut self, start: u32) -> SchemaExtension<'a> {
        self.expect_name_value("schema");
        let directives = self.parse_directives(Constness::Const);
        let root_operations = self.parse_root_operation_types_if_present();
        if directives.is_empty() && root_operations.is_empty() {
            self.err("expected Directives or Root Operation Types");
        }
        SchemaExtension { directives, root_operations, span: self.span_from(start) }
    }

    fn parse_root_operation_types_if_present(
        &mut self,
    ) -> ArenaVec<'a, RootOperationTypeDefinition<'a>> {
        if self.peek() != Some(T!['{']) {
            return ArenaVec::new_in(&self.allocator);
        }

        self.bump();
        let mut root_operations = ArenaVec::new_in(&self.allocator);
        self.peek_while(|parser, kind| match kind {
            T!['}'] => {
                parser.bump();
                ControlFlow::Break(())
            }
            TokenKind::Name => {
                root_operations.push(parser.parse_root_operation_type_definition());
                ControlFlow::Continue(())
            }
            TokenKind::Eof => {
                parser.err("expected }");
                ControlFlow::Break(())
            }
            _ => {
                parser.err_and_pop("expected Root Operation Type Definition");
                ControlFlow::Continue(())
            }
        });
        root_operations
    }

    fn parse_root_operation_type_definition(&mut self) -> RootOperationTypeDefinition<'a> {
        let start = self.current_start();
        let operation_type = self.parse_operation_type("expected an Operation Type");
        self.expect(T![:], "expected :");
        let named_type = self.parse_named_type().unwrap_or_else(|| self.missing_named_type());
        RootOperationTypeDefinition { operation_type, named_type, span: self.span_from(start) }
    }

    fn parse_directive_extension_from(&mut self, start: u32) -> DirectiveExtension<'a> {
        self.expect_name_value("directive");
        self.expect(T![@], "expected @ symbol");
        let name = self.parse_name().unwrap_or_else(|| self.missing_name());
        let directives = self.parse_directives(Constness::Const);
        if directives.is_empty() {
            self.err("expected Directives");
        }
        DirectiveExtension { name, directives, span: self.span_from(start) }
    }

    fn parse_directive_definition(
        &mut self,
        description: Option<ArenaBox<'a, StringValue<'a>>>,
    ) -> DirectiveDefinition<'a> {
        let start = self.definition_start(&description);
        self.expect_name_value("directive");
        self.expect(T![@], "expected @ symbol");
        let name = self.parse_name().unwrap_or_else(|| self.missing_name());
        let arguments = self.parse_arguments_definition_if_present();
        let directives = self.parse_directives(Constness::Const);
        let repeatable = if self.peek_data() == Some("repeatable") {
            self.bump();
            true
        } else {
            false
        };
        self.expect_name_value("on");
        let locations = self.parse_directive_locations();

        DirectiveDefinition {
            description,
            name,
            arguments,
            directives,
            repeatable,
            locations,
            span: self.span_from(start),
        }
    }

    fn parse_directive_locations(&mut self) -> ArenaVec<'a, DirectiveLocation<'a>> {
        if self.peek() == Some(T![|]) {
            self.bump();
        }

        let mut locations = ArenaVec::new_in(&self.allocator);
        loop {
            if let Some(token) = self.peek_token().copied()
                && token.kind() == TokenKind::Name
            {
                self.bump();
                locations.push(DirectiveLocation { name: token.data(), span: token_span(&token) });
            } else {
                self.err("expected valid Directive Location");
                break;
            }

            if self.peek() == Some(T![|]) {
                self.bump();
            } else {
                break;
            }
        }
        locations
    }

    fn parse_scalar_type_definition(
        &mut self,
        description: Option<ArenaBox<'a, StringValue<'a>>>,
    ) -> ScalarTypeDefinition<'a> {
        let start = self.definition_start(&description);
        self.expect_name_value("scalar");
        let name = self.parse_name().unwrap_or_else(|| self.missing_name());
        let directives = self.parse_directives(Constness::Const);
        ScalarTypeDefinition { description, name, directives, span: self.span_from(start) }
    }

    fn parse_scalar_type_extension_from(&mut self, start: u32) -> ScalarTypeExtension<'a> {
        self.expect_name_value("scalar");
        let name = self.parse_name().unwrap_or_else(|| self.missing_name());
        let directives = self.parse_directives(Constness::Const);
        if directives.is_empty() {
            self.err("expected Directives");
        }
        ScalarTypeExtension { name, directives, span: self.span_from(start) }
    }

    fn parse_object_type_definition(
        &mut self,
        description: Option<ArenaBox<'a, StringValue<'a>>>,
    ) -> ObjectTypeDefinition<'a> {
        let start = self.definition_start(&description);
        self.expect_name_value("type");
        let name = self.parse_name().unwrap_or_else(|| self.missing_name());
        let interfaces = self.parse_implements_interfaces();
        let directives = self.parse_directives(Constness::Const);
        let fields = self.parse_fields_definition_if_present();
        ObjectTypeDefinition {
            description,
            name,
            interfaces,
            directives,
            fields,
            span: self.span_from(start),
        }
    }

    fn parse_object_type_extension_from(&mut self, start: u32) -> ObjectTypeExtension<'a> {
        self.expect_name_value("type");
        let name = self.parse_name().unwrap_or_else(|| self.missing_name());
        let interfaces = self.parse_implements_interfaces();
        let directives = self.parse_directives(Constness::Const);
        let fields = self.parse_fields_definition_if_present();
        if interfaces.is_empty() && directives.is_empty() && fields.is_empty() {
            self.err("expected Implements Interfaces, Directives, or Fields Definition");
        }
        ObjectTypeExtension { name, interfaces, directives, fields, span: self.span_from(start) }
    }

    fn parse_interface_type_definition(
        &mut self,
        description: Option<ArenaBox<'a, StringValue<'a>>>,
    ) -> InterfaceTypeDefinition<'a> {
        let start = self.definition_start(&description);
        self.expect_name_value("interface");
        let name = self.parse_name().unwrap_or_else(|| self.missing_name());
        let interfaces = self.parse_implements_interfaces();
        let directives = self.parse_directives(Constness::Const);
        let fields = self.parse_fields_definition_if_present();
        InterfaceTypeDefinition {
            description,
            name,
            interfaces,
            directives,
            fields,
            span: self.span_from(start),
        }
    }

    fn parse_interface_type_extension_from(&mut self, start: u32) -> InterfaceTypeExtension<'a> {
        self.expect_name_value("interface");
        let name = self.parse_name().unwrap_or_else(|| self.missing_name());
        let interfaces = self.parse_implements_interfaces();
        let directives = self.parse_directives(Constness::Const);
        let fields = self.parse_fields_definition_if_present();
        if interfaces.is_empty() && directives.is_empty() && fields.is_empty() {
            self.err("expected an Implements Interfaces, Directives, or a Fields Definition");
        }
        InterfaceTypeExtension { name, interfaces, directives, fields, span: self.span_from(start) }
    }

    fn parse_implements_interfaces(&mut self) -> ArenaVec<'a, NamedType<'a>> {
        if self.peek_data() != Some("implements") {
            return ArenaVec::new_in(&self.allocator);
        }

        self.bump();
        if self.peek() == Some(T![&]) {
            self.bump();
        }

        let mut interfaces = ArenaVec::new_in(&self.allocator);
        loop {
            if let Some(named_type) = self.parse_named_type() {
                interfaces.push(named_type);
            } else {
                self.err("expected Implements Interface");
                break;
            }

            if self.peek() == Some(T![&]) {
                self.bump();
            } else {
                break;
            }
        }
        interfaces
    }

    fn parse_fields_definition_if_present(&mut self) -> ArenaVec<'a, FieldDefinition<'a>> {
        if self.peek() != Some(T!['{']) {
            return ArenaVec::new_in(&self.allocator);
        }

        self.bump();
        let mark = self.scratch_mark();
        self.peek_while(|parser, kind| match kind {
            T!['}'] => {
                if parser.scratch.len() == mark {
                    parser.err("expected Field Definition");
                }
                parser.bump();
                ControlFlow::Break(())
            }
            TokenKind::Name | TokenKind::StringValue => {
                let field = parser.parse_field_definition();
                parser.scratch.push(ScratchNode::FieldDefinition(field));
                ControlFlow::Continue(())
            }
            TokenKind::Eof => {
                parser.err("expected }");
                ControlFlow::Break(())
            }
            _ => {
                parser.err_and_pop("expected a Field Definition");
                ControlFlow::Continue(())
            }
        });
        self.drain_scratch(mark, |node| match node {
            ScratchNode::FieldDefinition(field) => field,
            _ => unreachable!("scratch stack discipline"),
        })
    }

    fn parse_field_definition(&mut self) -> FieldDefinition<'a> {
        let start = self.current_start();
        let description = self.parse_description_if_present();
        let name = self.parse_name().unwrap_or_else(|| self.missing_name());
        let arguments = self.parse_arguments_definition_if_present();
        let ty = if self.peek() == Some(T![:]) {
            self.bump();
            self.parse_type_inner()
        } else {
            self.err("expected a Type");
            None
        };
        let directives = self.parse_directives(Constness::Const);
        FieldDefinition {
            description,
            name,
            arguments,
            ty,
            directives,
            span: self.span_from(start),
        }
    }

    fn parse_arguments_definition_if_present(&mut self) -> ArenaVec<'a, InputValueDefinition<'a>> {
        if self.peek() != Some(T!['(']) {
            return ArenaVec::new_in(&self.allocator);
        }

        self.bump();
        let mark = self.scratch_mark();
        self.peek_while(|parser, kind| match kind {
            T![')'] => {
                parser.bump();
                ControlFlow::Break(())
            }
            TokenKind::Name | TokenKind::StringValue => {
                let definition = parser.parse_input_value_definition();
                parser.scratch.push(ScratchNode::InputValueDefinition(definition));
                ControlFlow::Continue(())
            }
            TokenKind::Eof => {
                parser.err("expected )");
                ControlFlow::Break(())
            }
            _ => {
                parser.err_and_pop("expected an Argument Definition");
                ControlFlow::Continue(())
            }
        });
        self.drain_scratch(mark, |node| match node {
            ScratchNode::InputValueDefinition(definition) => definition,
            _ => unreachable!("scratch stack discipline"),
        })
    }

    fn parse_input_value_definition(&mut self) -> InputValueDefinition<'a> {
        let start = self.current_start();
        let description = self.parse_description_if_present();
        let name = self.parse_name().unwrap_or_else(|| self.missing_name());
        let ty = if self.peek() == Some(T![:]) {
            self.bump();
            self.parse_type_inner()
        } else {
            self.err("expected a Type");
            None
        };
        let default_value = if self.peek() == Some(T![=]) {
            self.bump();
            Some(self.parse_value(Constness::Const, false))
        } else {
            None
        };
        let directives = self.parse_directives(Constness::Const);
        InputValueDefinition {
            description,
            name,
            ty,
            default_value,
            directives,
            span: self.span_from(start),
        }
    }

    fn parse_union_type_definition(
        &mut self,
        description: Option<ArenaBox<'a, StringValue<'a>>>,
    ) -> UnionTypeDefinition<'a> {
        let start = self.definition_start(&description);
        self.expect_name_value("union");
        let name = self.parse_name().unwrap_or_else(|| self.missing_name());
        let directives = self.parse_directives(Constness::Const);
        let members = self.parse_union_members_if_present();
        UnionTypeDefinition { description, name, directives, members, span: self.span_from(start) }
    }

    fn parse_union_type_extension_from(&mut self, start: u32) -> UnionTypeExtension<'a> {
        self.expect_name_value("union");
        let name = self.parse_name().unwrap_or_else(|| self.missing_name());
        let directives = self.parse_directives(Constness::Const);
        let members = self.parse_union_members_if_present();
        if directives.is_empty() && members.is_empty() {
            self.err("expected Directives or Union Member Types");
        }
        UnionTypeExtension { name, directives, members, span: self.span_from(start) }
    }

    fn parse_union_members_if_present(&mut self) -> ArenaVec<'a, NamedType<'a>> {
        if self.peek() != Some(T![=]) {
            return ArenaVec::new_in(&self.allocator);
        }

        self.bump();
        if self.peek() == Some(T![|]) {
            self.bump();
        }

        let mut members = ArenaVec::new_in(&self.allocator);
        loop {
            if let Some(member) = self.parse_named_type() {
                members.push(member);
            } else {
                self.err("expected Union Member Type");
                break;
            }

            if self.peek() == Some(T![|]) {
                self.bump();
            } else {
                break;
            }
        }
        members
    }

    fn parse_enum_type_definition(
        &mut self,
        description: Option<ArenaBox<'a, StringValue<'a>>>,
    ) -> EnumTypeDefinition<'a> {
        let start = self.definition_start(&description);
        self.expect_name_value("enum");
        let name = self.parse_name().unwrap_or_else(|| self.missing_name());
        let directives = self.parse_directives(Constness::Const);
        let values = self.parse_enum_values_definition_if_present();
        EnumTypeDefinition { description, name, directives, values, span: self.span_from(start) }
    }

    fn parse_enum_type_extension_from(&mut self, start: u32) -> EnumTypeExtension<'a> {
        self.expect_name_value("enum");
        let name = self.parse_name().unwrap_or_else(|| self.missing_name());
        let directives = self.parse_directives(Constness::Const);
        let values = self.parse_enum_values_definition_if_present();
        if directives.is_empty() && values.is_empty() {
            self.err("expected Directives or Enum Values Definition");
        }
        EnumTypeExtension { name, directives, values, span: self.span_from(start) }
    }

    fn parse_enum_values_definition_if_present(&mut self) -> ArenaVec<'a, EnumValueDefinition<'a>> {
        if self.peek() != Some(T!['{']) {
            return ArenaVec::new_in(&self.allocator);
        }

        self.bump();
        let mut values = ArenaVec::new_in(&self.allocator);
        self.peek_while(|parser, kind| match kind {
            T!['}'] => {
                if values.is_empty() {
                    parser.err("expected Enum Value Definition");
                }
                parser.bump();
                ControlFlow::Break(())
            }
            TokenKind::Name | TokenKind::StringValue => {
                values.push(parser.parse_enum_value_definition());
                ControlFlow::Continue(())
            }
            TokenKind::Eof => {
                parser.err("expected }");
                ControlFlow::Break(())
            }
            _ => {
                parser.err_and_pop("expected an Enum Value Definition");
                ControlFlow::Continue(())
            }
        });
        values
    }

    fn parse_enum_value_definition(&mut self) -> EnumValueDefinition<'a> {
        let start = self.current_start();
        let description = self.parse_description_if_present();
        let value = EnumValue { name: self.parse_name().unwrap_or_else(|| self.missing_name()) };
        if matches!(value.name.as_str(), "true" | "false" | "null") {
            self.err("invalid Enum Value");
        }
        let directives = self.parse_directives(Constness::Const);
        EnumValueDefinition { description, value, directives, span: self.span_from(start) }
    }

    fn parse_input_object_type_definition(
        &mut self,
        description: Option<ArenaBox<'a, StringValue<'a>>>,
    ) -> InputObjectTypeDefinition<'a> {
        let start = self.definition_start(&description);
        self.expect_name_value("input");
        let name = self.parse_name().unwrap_or_else(|| self.missing_name());
        let directives = self.parse_directives(Constness::Const);
        let fields = self.parse_input_fields_definition_if_present();
        InputObjectTypeDefinition {
            description,
            name,
            directives,
            fields,
            span: self.span_from(start),
        }
    }

    fn parse_input_object_type_extension_from(
        &mut self,
        start: u32,
    ) -> InputObjectTypeExtension<'a> {
        self.expect_name_value("input");
        let name = self.parse_name().unwrap_or_else(|| self.missing_name());
        let directives = self.parse_directives(Constness::Const);
        let fields = self.parse_input_fields_definition_if_present();
        if directives.is_empty() && fields.is_empty() {
            self.err("expected Directives or Input Fields Definition");
        }
        InputObjectTypeExtension { name, directives, fields, span: self.span_from(start) }
    }

    fn parse_input_fields_definition_if_present(
        &mut self,
    ) -> ArenaVec<'a, InputValueDefinition<'a>> {
        if self.peek() != Some(T!['{']) {
            return ArenaVec::new_in(&self.allocator);
        }

        self.bump();
        let mark = self.scratch_mark();
        self.peek_while(|parser, kind| match kind {
            T!['}'] => {
                if parser.scratch.len() == mark {
                    parser.err("expected an Input Value Definition");
                }
                parser.bump();
                ControlFlow::Break(())
            }
            TokenKind::Name | TokenKind::StringValue => {
                let field = parser.parse_input_value_definition();
                parser.scratch.push(ScratchNode::InputValueDefinition(field));
                ControlFlow::Continue(())
            }
            TokenKind::Eof => {
                parser.err("expected }");
                ControlFlow::Break(())
            }
            _ => {
                parser.err_and_pop("expected an Input Value Definition");
                ControlFlow::Continue(())
            }
        });
        self.drain_scratch(mark, |node| match node {
            ScratchNode::InputValueDefinition(field) => field,
            _ => unreachable!("scratch stack discipline"),
        })
    }

    fn parse_description_if_present(&mut self) -> Option<ArenaBox<'a, StringValue<'a>>> {
        if self.peek() == Some(TokenKind::StringValue) {
            let value = self.parse_string_value()?;
            Some(ArenaBox::new_in(value, &self.allocator))
        } else {
            None
        }
    }

    /// The span start of a definition: the start of its already-parsed
    /// description if it has one, otherwise the start of the current token.
    fn definition_start(&mut self, description: &Option<ArenaBox<'a, StringValue<'a>>>) -> u32 {
        description.as_ref().map_or_else(|| self.current_start(), |value| value.span.start)
    }

    fn parse_string_value(&mut self) -> Option<StringValue<'a>> {
        let token = self.bump()?;
        let raw = token.data();
        let block = raw.starts_with(r#"""""#);
        let value = if block {
            let content = raw
                .strip_prefix(r#"""""#)
                .and_then(|value| value.strip_suffix(r#"""""#))
                .unwrap_or(raw);
            if content.contains('\r') {
                self.allocator.alloc_str(&normalize_block_string(raw))
            } else {
                // No line endings to normalize: borrow from the source text.
                content
            }
        } else {
            // Strip exactly one quote from each end: `trim_matches` would also
            // eat a trailing escaped quote (`"abc\""` must keep its `"`).
            let content =
                raw.strip_prefix('"').and_then(|value| value.strip_suffix('"')).unwrap_or(raw);
            if content.contains('\\') {
                self.allocator.alloc_str(&unescape_string(content))
            } else {
                // No escape sequences: borrow from the source text.
                content
            }
        };
        Some(StringValue { raw, value, block, span: token_span(&token) })
    }

    fn parse_name(&mut self) -> Option<Name<'a>> {
        if self.peek()? != TokenKind::Name {
            self.err("expected a Name");
            return None;
        }
        let token = self.bump().expect("peeked Name token must be available");
        Some(Name { value: token.data(), span: token_span(&token) })
    }

    fn expect_name_value(&mut self, expected: &str) {
        if self.peek_data() == Some(expected) {
            self.bump();
        } else {
            self.err(&format!("expected {expected}"));
        }
    }

    fn expect(&mut self, token: TokenKind, message: &str) {
        if self.peek() == Some(token) {
            self.bump();
        } else {
            self.err(message);
        }
    }

    fn missing_name(&self) -> Name<'a> {
        Name { value: "", span: Span::new(self.last_end, self.last_end) }
    }

    fn missing_named_type(&self) -> NamedType<'a> {
        NamedType { name: self.missing_name() }
    }

    fn missing_variable(&self) -> Variable<'a> {
        Variable { name: self.missing_name(), span: Span::new(self.last_end, self.last_end) }
    }

    fn limit_err<S: Into<String>>(&mut self, message: S) {
        let index = if let Some(token) = self.peek_token() {
            token.index()
        } else {
            self.last_end as usize
        };
        self.push_err(Error::limit(message, index));
        self.accept_errors = false;
    }

    fn err(&mut self, message: &str) {
        let Some(token) = self.peek_token().copied() else {
            return;
        };
        let err = if token.kind() == TokenKind::Eof {
            Error::eof(message, token.index())
        } else {
            Error::with_loc(message, token.data().to_string(), token.index())
        };
        self.push_err(err);
    }

    fn err_and_pop(&mut self, message: &str) {
        let Some(token) = self.bump() else {
            return;
        };
        let err = if token.kind() == TokenKind::Eof {
            Error::eof(message, token.index())
        } else {
            Error::with_loc(message, token.data().to_string(), token.index())
        };
        self.push_err(err);
    }

    fn push_err(&mut self, err: Error) {
        if self.accept_errors {
            self.errors.push(err);
        }
    }

    fn peek_while(&mut self, mut run: impl FnMut(&mut Parser<'a>, TokenKind) -> ControlFlow<()>) {
        while let Some(kind) = self.peek() {
            let before = self.current_token;
            match run(self, kind) {
                ControlFlow::Break(()) => break,
                ControlFlow::Continue(()) => {
                    debug_assert!(
                        before != self.current_token,
                        "peek_while() iteration must advance parsing"
                    );
                }
            }
        }
    }

    fn peek(&mut self) -> Option<TokenKind> {
        self.peek_token().map(Token::kind)
    }

    fn peek_data(&mut self) -> Option<&'a str> {
        self.peek_token().map(Token::data)
    }

    fn peek_token(&mut self) -> Option<&Token<'a>> {
        if self.current_token.is_none() {
            self.current_token = self.next_significant_token();
        }
        self.current_token.as_ref()
    }

    fn bump(&mut self) -> Option<Token<'a>> {
        let token = if let Some(token) = self.current_token.take() {
            token
        } else {
            self.next_significant_token()?
        };
        self.last_end = span_index(token.index() + token.data().len());
        Some(token)
    }

    fn next_significant_token(&mut self) -> Option<Token<'a>> {
        // `next_significant` skips whitespace and comma trivia in the cursor;
        // comments still surface as tokens so their spans can be recorded.
        loop {
            match self.lexer.next_significant()? {
                Ok(token) => match token.kind() {
                    TokenKind::Comment => {
                        let span = token_span(&token);
                        self.comments.push(span);
                    }
                    _ => return Some(token),
                },
                Err(err) => {
                    if err.is_limit() {
                        self.accept_errors = false;
                    }
                    self.errors.push(err);
                }
            }
        }
    }

    fn current_start(&mut self) -> u32 {
        if let Some(token) = self.peek_token() { span_index(token.index()) } else { self.last_end }
    }

    fn current_span(&mut self) -> Span {
        self.peek_token().map(token_span).unwrap_or_else(|| Span::new(self.last_end, self.last_end))
    }

    fn span_from(&self, start: u32) -> Span {
        Span::new(start, self.last_end.max(start))
    }
}

/// Converts a byte index to a span offset.
///
/// `Parser::new` asserts the source text fits in `u32`, so token indexes are
/// always in range.
#[expect(clippy::cast_possible_truncation)]
#[inline]
fn span_index(index: usize) -> u32 {
    debug_assert!(u32::try_from(index).is_ok());
    index as u32
}

fn token_span(token: &Token<'_>) -> Span {
    let start = span_index(token.index());
    let end = span_index(token.index() + token.data().len());
    Span::new(start, end)
}

fn unescape_string(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut iter = input.chars();
    while let Some(c) = iter.next() {
        if c != '\\' {
            output.push(c);
            continue;
        }

        let Some(c2) = iter.next() else {
            output.push(c);
            break;
        };

        match c2 {
            '"' | '\\' | '/' => output.push(c2),
            'b' => output.push('\u{0008}'),
            'f' => output.push('\u{000c}'),
            'n' => output.push('\n'),
            'r' => output.push('\r'),
            't' => output.push('\t'),
            'u' => {
                let value = iter.by_ref().take(4).fold(0, |acc, c| {
                    let digit = c.to_digit(16).unwrap_or(0);
                    (acc << 4) + digit
                });
                if let Some(c) = char::from_u32(value) {
                    output.push(c);
                }
            }
            _ => {}
        }
    }
    output
}

fn normalize_block_string(raw: &str) -> String {
    let content =
        raw.strip_prefix(r#"""""#).and_then(|value| value.strip_suffix(r#"""""#)).unwrap_or(raw);
    let mut output = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\r' {
            chars.next_if_eq(&'\n');
            output.push('\n');
        } else {
            output.push(ch);
        }
    }
    output
}
