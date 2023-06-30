use std::{env, fs, rc::Rc, sync::Arc};

use oxc_diagnostics::miette::{miette, LabeledSpan};
use oxc_semantic::Semantic;
use trustfall::{execute_query, Schema, TryIntoStruct};

use crate::{
    context::LintContext,
    lint_adapter::{InputQuery, LintAdapter},
};

#[derive(Debug, serde::Deserialize)]
struct SpanInfo {
    span_start: u64,
    span_end: u64,
}

pub struct LinterPlugin {
    rules: Vec<InputQuery>,
    schema: Schema,
}

impl LinterPlugin {
    pub fn new() -> Self {
        let queries_path = env::var("OXC_PLUGIN").unwrap();
        let rules = fs::read_dir(queries_path)
            .unwrap()
            .filter_map(std::result::Result::ok)
            .filter(|dir_entry| dir_entry.path().is_file())
            .filter(|f| {
                std::path::Path::new(f.path().as_os_str().to_str().unwrap())
                    .extension()
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("ron"))
            })
            .map(|f| fs::read_to_string(f.path()))
            .map(std::result::Result::unwrap)
            .map(|rule| ron::from_str::<InputQuery>(rule.as_str()).unwrap())
            .collect::<Vec<_>>();

        let schema_file = std::include_str!("schema.graphql");
        // TODO: parse this at compile time
        let schema = Schema::parse(schema_file).expect("schema file failed to parse");
        Self { rules, schema }
    }

    pub fn run(&self, ctx: &mut LintContext, semantic: &Rc<Semantic<'_>>) {
        let inner = LintAdapter { semantic: Rc::clone(semantic) };
        let adapter = Arc::from(&inner);
        for input_query in &self.rules {
            for data_item in execute_query(
                &self.schema,
                Arc::clone(&adapter),
                &input_query.query,
                input_query.args.clone(),
            )
            .expect(
                format!("not a legal query in query: \n\n\n{}", input_query.query.as_str())
                    .as_str(),
            )
            .map(|v| {
                v.try_into_struct::<SpanInfo>().expect(
                    "Could not deserialize query results into SpanInfo{span_start, span_end}",
                )
            })
            .take(usize::MAX)
            {
                ctx.with_rule_name("a rule");
                let SpanInfo { span_start: start, span_end: end } = data_item;
                // TODO: this isn't how we do this at all, need to make this consistent with the project's miette style
                let c = miette!(
                    labels = vec![LabeledSpan::at(
                        (start as usize, (end - start) as usize),
                        input_query.reason.as_str()
                    )],
                    "Unexpected error"
                );
                ctx.diagnostic(c);
            }
        }
    }
}
