use std::path::{Path, PathBuf};

use clap::{Arg, ArgMatches, Command};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_type_synthesis::synthesize_program;

use crate::{runner::Runner, CliRunResult};

const PRELUDE: &str = "
type StringOrNumber = string | number;

interface Operators {
    Add<T extends StringOrNumber, U extends StringOrNumber>(a: T, b: U): (T extends string ? string : U extends string ? string: number) & Ezno.ConstantFunction<'add'>;

    Mul(a: number, b: number): number & Ezno.ConstantFunction<'mul'>;

    StrictEqual(a: any, b: any): boolean & Ezno.ConstantFunction<'equal'>;
}

interface Math {
    sin(x: number): number & Ezno.ConstantFunction<'sin'>;
}

interface string {
    toUppercase(): string & Ezno.ConstantFunction<'uppercase'>
}

interface Console {
    log(msg: any): void;
}

declare var Math: Math;
declare var console: Console;
";

/// TODO temp
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct TypeCheckOptions {
    pub path: PathBuf,
    // TODO temp, for exhibition
    pub print_expression_mappings: bool,
    // TODO temp, for exhibition
    pub print_called_functions: bool,
}

#[allow(clippy::fallible_impl_from)]
impl<'a> From<&'a ArgMatches> for TypeCheckOptions {
    fn from(matches: &'a ArgMatches) -> Self {
        Self {
            path: PathBuf::from(matches.get_one::<String>("path").unwrap()),
            print_called_functions: matches.contains_id("print_called_functions"),
            print_expression_mappings: matches.contains_id("print_expression_mappings"),
        }
    }
}

pub struct TypeCheckRunner {
    options: TypeCheckOptions,
}

impl Runner for TypeCheckRunner {
    const ABOUT: &'static str =
        "NOTE: Experimental / work in progress. Check source code for type errors using Ezno";
    const NAME: &'static str = "check";

    fn new(matches: &ArgMatches) -> Self {
        let options = TypeCheckOptions::from(matches);
        Self { options }
    }

    fn init_command() -> Command {
        Command::new(Self::NAME)
            .arg(
                Arg::new("path")
                    .value_name("PATH")
                    .num_args(1)
                    .required(true)
                    .help("File to type check"),
            )
            .arg(
                Arg::new("print_expression_mappings")
                    .required(false)
                    .help("Print types of expressions"),
            )
            .arg(Arg::new("print_called_functions").required(false).help("Print called functions"))
    }

    /// # Panics
    fn run(&self) -> CliRunResult {
        let now = std::time::Instant::now();

        let path = Path::new(&self.options.path);
        let source_text = PRELUDE.to_owned()
            + &std::fs::read_to_string(path)
                .unwrap_or_else(|_| panic!("{} not found", self.options.path.display()));
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(path).unwrap();

        let ret = Parser::new(&allocator, &source_text, source_type).parse();

        if ret.errors.is_empty() {
            let (diagnostics, _events, types, mappings, root_env) =
                synthesize_program(&ret.program, |_: &std::path::Path| None);

            let duration = now.elapsed();

            if self.options.print_expression_mappings {
                let mappings = mappings.print_type_mappings(
                    &source_text,
                    &root_env.into_general_environment(),
                    &types,
                );
                eprintln!("{mappings}");
            }
            if self.options.print_called_functions {
                let called_functions = mappings.print_called_functions(&source_text);
                eprintln!("{called_functions}");
            }

            // TODO
            let number_of_diagnostics = 0;
            type_check_output::print_diagnostics_container(
                diagnostics,
                self.options.path.display().to_string(),
                source_text.clone(),
            );
            CliRunResult::TypeCheckResult { duration, number_of_diagnostics }
        } else {
            let duration = now.elapsed();
            let number_of_diagnostics = ret.errors.len();
            for error in ret.errors {
                let error = error.with_source_code(source_text.clone());
                println!("{error:?}");
            }
            CliRunResult::TypeCheckResult { duration, number_of_diagnostics }
        }
    }
}

mod type_check_output {
    use std::iter;

    use codespan_reporting::{
        diagnostic::{Diagnostic, Label, Severity},
        files::SimpleFile,
        term::{
            termcolor::{ColorChoice, StandardStream},
            Config,
        },
    };
    use oxc_type_synthesis::{
        Diagnostic as TypeCheckDiagnostic, DiagnosticKind, DiagnosticsContainer,
    };

    #[allow(clippy::items_after_statements)]
    pub(super) fn print_diagnostics_container(
        diagnostic_container: DiagnosticsContainer,
        path: String,
        content: String,
    ) {
        let files = SimpleFile::new(path, content);
        // let mut file_id_to_source_id = HashMap::<EznoSourceId, usize>::new();

        // Handling adding filename-file id mappings
        // for source_id in error_handler.sources() {
        //     let (filename, file_content) = source_id.get_file().unwrap();
        //     let name =
        //         filename.strip_prefix(env::current_dir().unwrap()).unwrap_or(&filename).to_owned();
        //     let file_id = files.add(name.display().to_string(), file_content);
        //     file_id_to_source_id.insert(source_id, file_id);
        // }

        let writer = StandardStream::stderr(ColorChoice::Always);
        let mut lock = writer.lock();

        for diagnostic in diagnostic_container.into_iter().rev() {
            // Conversion from Ezno -> codespan
            let diagnostic = match diagnostic {
                TypeCheckDiagnostic::Global { reason, kind } => Diagnostic {
                    severity: ezno_diagnostic_to_severity(&kind),
                    code: None,
                    message: reason,
                    labels: Vec::new(),
                    notes: Vec::default(),
                },
                TypeCheckDiagnostic::Position { reason, position, kind } => Diagnostic {
                    severity: ezno_diagnostic_to_severity(&kind),
                    code: None,
                    message: String::default(),
                    labels: vec![Label::primary((), position).with_message(reason)],
                    notes: Vec::default(),
                },
                TypeCheckDiagnostic::PositionWithAdditionLabels {
                    reason,
                    position,
                    labels,
                    kind,
                } => {
                    let (labels, notes) =
                        labels.into_iter().partition::<Vec<_>, _>(|(_, value)| value.is_some());

                    Diagnostic {
                        severity: ezno_diagnostic_to_severity(&kind),
                        code: None,
                        message: String::default(),
                        labels: iter::once(Label::primary((), position).with_message(reason))
                            .chain(labels.into_iter().map(|(message, position)| {
                                let position = position.unwrap();
                                Label::secondary((), position).with_message(message)
                            }))
                            .collect(),
                        notes: notes.into_iter().map(|(message, _)| message).collect(),
                    }
                }
            };

            codespan_reporting::term::emit(&mut lock, &Config::default(), &files, &diagnostic)
                .unwrap();
        }

        fn ezno_diagnostic_to_severity(kind: &DiagnosticKind) -> Severity {
            match kind {
                DiagnosticKind::Error => Severity::Error,
                DiagnosticKind::Warning => Severity::Warning,
                DiagnosticKind::Info => Severity::Note,
            }
        }
    }
}

// struct MietteEznoDiagnostic {
//     diagnostic: EznoDiagnostic,
//     severity: miette::Severity,
//     // TODO temp
//     source: &'static str,
// }

// impl std::fmt::Debug for MietteEznoDiagnostic {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("MietteEznoDiagnostic").field("diagnostic", &"..").finish()
//     }
// }

// impl std::fmt::Display for MietteEznoDiagnostic {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.write_str(self.diagnostic.reason())
//     }
// }
// impl std::error::Error for MietteEznoDiagnostic {}

// impl miette::Diagnostic for MietteEznoDiagnostic {
//     fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
//         None
//     }

//     fn severity(&self) -> Option<miette::Severity> {
//         Some(self.severity)
//     }

//     fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
//         None
//     }

//     fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
//         None
//     }

//     fn source_code(&self) -> Option<&dyn miette::SourceCode> {
//         match self.diagnostic {
//             EznoDiagnostic::Global(_) => None,
//             EznoDiagnostic::Position { reason: _, ref pos }
//             | EznoDiagnostic::PositionWithAdditionLabels { reason: _, labels: _, ref pos } => {
//                 // TODO temp
//                 None
//             }
//         }
//     }

//     fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
//         match self.diagnostic {
//             // TODO temp
//             EznoDiagnostic::Global(ref reason) => {
//                 Some(Box::new(iter::once(miette::LabeledSpan::new(Some(reason.clone()), 0, 0))))
//             }
//             EznoDiagnostic::Position { ref reason, ref pos } => {
//                 Some(Box::new(iter::once(miette::LabeledSpan::new(
//                     Some(reason.clone()),
//                     pos.start as usize,
//                     pos.end as usize - pos.start as usize,
//                 ))))
//             }
//             EznoDiagnostic::PositionWithAdditionLabels { ref reason, ref labels, ref pos } => {
//                 Some(Box::new(
//                     iter::once(miette::LabeledSpan::new(
//                         Some(reason.clone()),
//                         pos.start as usize,
//                         pos.end as usize - pos.start as usize,
//                     ))
//                     .chain(labels.iter().map(|(label, pos)| {
//                         if let Some(pos) = pos {
//                             miette::LabeledSpan::new(
//                                 Some(label.clone()),
//                                 pos.start as usize,
//                                 pos.end as usize - pos.start as usize,
//                             )
//                         } else {
//                             miette::LabeledSpan::new(Some(label.clone()), 0, 0)
//                         }
//                     })),
//                 ))
//             }
//         }
//     }

//     fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn miette::Diagnostic> + 'a>> {
//         None
//     }

//     fn diagnostic_source(&self) -> Option<&dyn miette::Diagnostic> {
//         None
//     }
// }
