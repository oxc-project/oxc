use std::{borrow::Cow, env, fs, path::Path, rc::Rc, sync::Arc};

use oxc_diagnostics::miette::{miette, LabeledSpan};
use oxc_semantic::Semantic;
use trustfall::{execute_query, Schema, TryIntoStruct};

use crate::{
    context::LintContext,
    lint_adapter::{InputQuery, LintAdapter},
};

enum SpanInfo {
    SingleSpanInfo(SingleSpanInfo),
    MultipleSpanInfo(MultipleSpanInfo),
}

#[derive(Debug, serde::Deserialize)]
struct SingleSpanInfo {
    span_start: u64,
    span_end: u64,
}

#[derive(Debug, serde::Deserialize)]
struct MultipleSpanInfo {
    span_start: Box<[u64]>,
    span_end: Box<[u64]>,
}

pub struct LinterPlugin {
    rules: Vec<InputQuery>,
    schema: Schema,
}

impl LinterPlugin {
    pub fn new() -> Self {
        let queries_path = env::var("OXC_PLUGIN").unwrap();
        let rules = fs::read_dir(queries_path)
            .expect("to readdir queries_path folder")
            .filter_map(std::result::Result::ok)
            .filter(|dir_entry| dir_entry.path().is_dir())
            .filter(|dir| !dir.path().to_str().map_or_else(|| true, |f| f.contains("ignore_")))
            .filter_map(|folder| fs::read_dir(folder.path()).ok())
            .flat_map(std::iter::IntoIterator::into_iter)
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

    pub fn run(&self, ctx: &mut LintContext, semantic: &Rc<Semantic<'_>>, path: Cow<'_, Path>) {
        let inner = LintAdapter { semantic: Rc::clone(semantic), path: path.to_path_buf() };
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
                if env::var("OXC_PRINT_TFALL_OUTPUTS").unwrap_or_else(|_| "false".to_owned())
                    == "true"
                {
                    println!("{v:#?}");
                }
                let multi = v.clone().try_into_struct::<MultipleSpanInfo>();
                let single = v.try_into_struct::<SingleSpanInfo>();
                single.map_or_else(
                    |_| SpanInfo::MultipleSpanInfo(multi.unwrap()),
                    SpanInfo::SingleSpanInfo,
                )
            })
            .take(usize::MAX)
            {
                ctx.with_rule_name("a rule");
                // TODO: this isn't how we do this at all, need to make this consistent with the project's miette style
                match data_item {
                    SpanInfo::SingleSpanInfo(SingleSpanInfo {
                        span_start: start,
                        span_end: end,
                    }) => {
                        ctx.diagnostic(miette!(
                            labels = vec![LabeledSpan::at(
                                (start as usize, (end - start) as usize),
                                input_query.reason.as_str()
                            )],
                            "Unexpected error"
                        ));
                    }
                    SpanInfo::MultipleSpanInfo(MultipleSpanInfo {
                        span_start: start,
                        span_end: end,
                    }) => {
                        for i in 0..start.len() {
                            ctx.diagnostic(miette!(
                                labels = vec![LabeledSpan::at(
                                    (start[i] as usize, (end[i] - start[i]) as usize),
                                    input_query.reason.as_str()
                                )],
                                "Unexpected error"
                            ));
                        }
                    }
                }
            }
        }
    }
}
