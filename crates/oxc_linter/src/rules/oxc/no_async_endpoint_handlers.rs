use std::ops::Deref;

use oxc_ast::{
    ast::{Argument, ArrowFunctionExpression, Expression, Function},
    AstKind,
};
use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use serde_json::Value;

use crate::{context::LintContext, rule::Rule, utils, AstNode};

#[derive(Debug, Default, Clone)]
pub struct NoAsyncEndpointHandlers(Box<NoAsyncEndpointHandlersConfig>);
impl Deref for NoAsyncEndpointHandlers {
    type Target = NoAsyncEndpointHandlersConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, Clone)]
pub struct NoAsyncEndpointHandlersConfig {
    allowed_names: Vec<CompactStr>,
}

pub fn no_async_handlers(
    function_span: Span,
    registered_span: Option<Span>,
    name: Option<&str>,
) -> OxcDiagnostic {
    #[allow(clippy::cast_possible_truncation)]
    const ASYNC_LEN: u32 = "async".len() as u32;

    // Only cover "async" in "async function (req, res) {}" or "async (req, res) => {}"
    let async_span = Span::sized(function_span.start, ASYNC_LEN);

    let labels: &[LabeledSpan] = match (registered_span, name) {
        // handler is declared separately from registration
        // `async function foo(req, res) {}; app.get('/foo', foo);`
        (Some(span), Some(name)) => &[
            async_span.label(format!("Async handler '{name}' is declared here")),
            span.primary_label("and is registered here"),
        ],
        // Shouldn't happen, since separate declaration/registration requires an
        // identifier to be bound
        (Some(span), None) => &[
            async_span.label("Async handler is declared here"),
            span.primary_label("and is registered here"),
        ],
        // `app.get('/foo', async function foo(req, res) {});`
        (None, Some(name)) => &[async_span.label(format!("Async handler '{name}' is used here"))],

        // `app.get('/foo', async (req, res) => {});`
        (None, None) => &[async_span.label("Async handler is used here")],
    };

    OxcDiagnostic::warn("Express endpoint handlers should not be async.")
        .with_labels(labels.iter().cloned())
        .with_help("Express <= 4.x does not handle Promise rejections. Use `new Promise((resolve, reject) => { ... }).catch(next)` instead.")
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the use of `async` functions as Express endpoint handlers.
    ///
    /// ### Why is this bad?
    ///
    /// Before v5, Express will not automatically handle Promise rejections from
    /// handler functions with your application's error handler. You must
    /// instead explicitly pass the rejected promise to `next()`.
    /// ```js
    /// const app = express()
    /// app.get('/', (req, res, next) => {
    ///   new Promise((resolve, reject) => {
    ///       return User.findById(req.params.id)
    ///   })
    ///     .then(user => res.json(user))
    ///     .catch(next)
    /// })
    /// ```
    ///
    /// If this is not done, your server will crash with an unhandled promise
    /// rejection.
    /// ```js
    /// const app = express()
    /// app.get('/', async (req, res) => {
    ///   // Server will crash if User.findById rejects
    ///   const user = await User.findById(req.params.id)
    ///   res.json(user)
    /// })
    /// ```
    ///
    /// See [Express' Error Handling
    /// Guide](https://expressjs.com/en/guide/error-handling.html) for more
    /// information.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const app = express();
    /// app.get('/', async (req, res) => {
    ///   const user = await User.findById(req.params.id);
    ///   res.json(user);
    /// });
    ///
    /// const router = express.Router();
    /// router.use(async (req, res, next) => {
    ///   const user = await User.findById(req.params.id);
    ///   req.user = user;
    ///   next();
    /// });
    ///
    /// const createUser = async (req, res) => {
    ///   const user = await User.create(req.body);
    ///   res.json(user);
    /// }
    /// app.post('/user', createUser);
    ///
    /// // Async handlers that are imported will not be detected because each
    /// // file is checked in isolation. This does not trigger the rule, but still
    /// // violates it and _will_ result in server crashes.
    /// const asyncHandler = require('./asyncHandler');
    /// app.get('/async', asyncHandler);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const app = express();
    /// // not async
    /// app.use((req, res, next) => {
    ///   req.receivedAt = Date.now();
    /// })
    ///
    /// app.get('/', (req, res, next) => {
    ///   fs.readFile('/file-does-not-exist', (err, data) => {
    ///     if (err) {
    ///       next(err) // Pass errors to Express.
    ///     } else {
    ///       res.send(data)
    ///     }
    ///   })
    /// })
    ///
    /// const asyncHandler = async (req, res) => {
    ///   const user = await User.findById(req.params.id);
    ///   res.json(user);
    /// }
    /// app.get('/user', (req, res, next) => asyncHandler(req, res).catch(next))
    /// ```
    ///
    /// ## Configuration
    ///
    /// This rule takes the following configuration:
    /// ```ts
    /// type NoAsyncEndpointHandlersConfig = {
    ///   /**
    ///    * An array of names that are allowed to be async.
    ///    */
    ///   allowedNames?: string[];
    /// }
    /// ```
    NoAsyncEndpointHandlers,
    suspicious
);

impl Rule for NoAsyncEndpointHandlers {
    fn from_configuration(value: Value) -> Self {
        let mut allowed_names: Vec<CompactStr> = value
            .get(0)
            .and_then(Value::as_object)
            .and_then(|config| config.get("allowedNames"))
            .and_then(Value::as_array)
            .map(|names| names.iter().filter_map(Value::as_str).map(CompactStr::from).collect())
            .unwrap_or_default();
        allowed_names.sort_unstable();
        allowed_names.dedup();

        Self(Box::new(NoAsyncEndpointHandlersConfig { allowed_names }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let kind = node.kind();
        let Some((_endpoint, args)) = utils::as_endpoint_registration(&kind) else {
            return;
        };
        for arg in
            args.iter().filter_map(Argument::as_expression).map(Expression::get_inner_expression)
        {
            self.check_endpoint_arg(ctx, arg);
        }
    }
}

impl NoAsyncEndpointHandlers {
    fn check_endpoint_arg<'a>(&self, ctx: &LintContext<'a>, arg: &Expression<'a>) {
        self.check_endpoint_expr(ctx, None, None, arg);
    }

    fn check_endpoint_expr<'a>(
        &self,
        ctx: &LintContext<'a>,
        id_name: Option<&str>,
        registered_at: Option<Span>,
        arg: &Expression<'a>,
    ) {
        match arg {
            Expression::Identifier(handler) => {
                // Unresolved reference? Nothing we can do.
                let Some(symbol_id) = handler
                    .reference_id()
                    .and_then(|id| ctx.symbols().get_reference(id).symbol_id())
                else {
                    return;
                };

                // Cannot check imported handlers without cross-file analysis.
                let flags = ctx.symbols().get_flags(symbol_id);
                if flags.is_import() {
                    return;
                }

                let decl_id = ctx.symbols().get_declaration(symbol_id);
                let decl_node = ctx.nodes().get_node(decl_id);
                let registered_at = registered_at.or(Some(handler.span));
                match decl_node.kind() {
                    AstKind::Function(f) => self.check_function(ctx, registered_at, id_name, f),
                    AstKind::VariableDeclarator(decl) => {
                        if let Some(init) = &decl.init {
                            if let Expression::Identifier(id) = &init {
                                if decl
                                    .id
                                    .get_identifier()
                                    .is_some_and(|declared| declared == id.name)
                                {
                                    return;
                                }
                            }
                            self.check_endpoint_expr(ctx, id_name, registered_at, init);
                        }
                    }
                    _ => {}
                }
            }
            func if utils::is_endpoint_handler(func) => {
                match func {
                    // `app.get('/', (async?) function (req, res) {}`
                    Expression::FunctionExpression(f) => {
                        self.check_function(ctx, registered_at, id_name, f);
                    }
                    Expression::ArrowFunctionExpression(f) => {
                        self.check_arrow(ctx, registered_at, id_name, f);
                    }
                    _ => unreachable!(),
                }
            }
            _ => {}
        }
    }

    fn check_function<'a>(
        &self,
        ctx: &LintContext<'a>,
        registered_at: Option<Span>,
        id_name: Option<&str>,
        f: &Function<'a>,
    ) {
        if !f.r#async {
            return;
        }

        let name = f.name().map(|n| n.as_str()).or(id_name);
        if name.is_some_and(|name| self.is_allowed_name(name)) {
            return;
        }

        ctx.diagnostic(no_async_handlers(f.span, registered_at, name));
    }

    fn check_arrow<'a>(
        &self,
        ctx: &LintContext<'a>,
        registered_at: Option<Span>,
        id_name: Option<&str>,
        f: &ArrowFunctionExpression<'a>,
    ) {
        if !f.r#async {
            return;
        }
        if id_name.is_some_and(|name| self.is_allowed_name(name)) {
            return;
        }

        ctx.diagnostic(no_async_handlers(f.span, registered_at, id_name));
    }

    fn is_allowed_name(&self, name: &str) -> bool {
        self.allowed_names.binary_search_by(|allowed| allowed.as_str().cmp(name)).is_ok()
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        ("app.get('/', fooController)", None),
        ("app.get('/', (req, res) => {})", None),
        ("app.get('/', (req, res) => {})", None),
        ("app.get('/', function (req, res) {})", None),
        ("app.get('/', middleware, function (req, res) {})", None),
        ("app.get('/', (req, res, next) => {})", None),
        ("app.get('/', (err, req, res, next) => {})", None),
        ("app.get('/', (err, req, res) => {})", None),
        ("app.get('/', (err, req, res) => {})", None),
        ("app.get('/', (req, res) => Promise.resolve())", None),
        ("app.get('/', (req, res) => new Promise((resolve, reject) => resolve()))", None),
        ("app.use(middleware)", None),
        ("app.get(middleware)", None),
        (
            "function ctl(req, res) {}
             app.get(ctl)",
            None,
        ),
        ("weirdName.get('/', async () =>  {})", None),
        ("weirdName.get('/', async (notARequestObject) =>  {})", None),
        // allowed names
        (
            "async function ctl(req, res) {}
             app.get(ctl)",
            Some(json!([ { "allowedNames": ["ctl"] } ])),
        ),
        (
            "
            async function middleware(req, res, next) {}
            app.use(middleware)
            ",
            Some(json!([ { "allowedNames": ["middleware"] } ])),
        ),
        // https://github.com/oxc-project/oxc/issues/6583
        (
            "
            class B{o(a={}){const attribute=attribute
            c.get(attribute)}}
            ",
            None,
        ),
    ];

    let fail = vec![
        ("app.get('/', async function (req, res) {})", None),
        ("app.get('/', async (req, res) =>  {})", None),
        ("app.get('/', async (req, res, next) =>  {})", None),
        ("weirdName.get('/', async (req, res) =>  {})", None),
        ("weirdName.get('/', async (req, res) =>  {})", None),
        (
            "
            async function foo(req, res) {}
            app.post('/', foo)
            ",
            None,
        ),
        (
            "
            const foo = async (req, res) => {}
            app.post('/', foo)
            ",
            None,
        ),
        (
            "
            async function middleware(req, res, next) {}
            app.use(middleware)
            ",
            None,
        ),
        (
            "
            async function foo(req, res) {}
            const bar = foo;
            app.post('/', bar)
            ",
            None,
        ),
    ];

    Tester::new(NoAsyncEndpointHandlers::NAME, pass, fail).test_and_snapshot();
}
