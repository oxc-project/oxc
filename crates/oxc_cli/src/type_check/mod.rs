use std::path::{Path, PathBuf};

use clap::{Arg, ArgMatches, Command};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_type_synthesis::synthesize_program;

use crate::CliRunResult;

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
}

#[allow(clippy::fallible_impl_from)]
impl<'a> From<&'a ArgMatches> for TypeCheckOptions {
    fn from(matches: &'a ArgMatches) -> Self {
        Self { path: PathBuf::from(matches.get_one::<String>("path").unwrap()) }
    }
}

pub fn type_check_command() -> Command {
    Command::new("type-check")
        .about(
            "NOTE: Experimental / work in progress. Check source code for type errors using Ezno",
        )
        .arg(Arg::new("path").value_name("PATH").num_args(1..).help("File to type check"))
}

pub struct TypeCheckRunner {
    options: TypeCheckOptions,
}

impl TypeCheckRunner {
    pub fn new(options: TypeCheckOptions) -> Self {
        Self { options }
    }

    /// # Panics
    pub fn run(&self) -> CliRunResult {
        let now = std::time::Instant::now();

        let path = Path::new(&self.options.path);
        let source_text = PRELUDE.to_owned()
            + &std::fs::read_to_string(path)
                .unwrap_or_else(|_| panic!("{} not found", self.options.path.display()));
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(path).unwrap();

        let ret = Parser::new(&allocator, &source_text, source_type).parse();

        if ret.errors.is_empty() {
            let (diagnostics, _events, _types) =
                synthesize_program(&ret.program, |_: &std::path::Path| None);

            let duration = now.elapsed();

            // if args.iter().any(|arg| arg == "--types") {
            //     eprintln!("Types:");
            //     for item in types {
            //         eprintln!("\t{:?}", item);
            //     }
            // }
            // if args.iter().any(|arg| arg == "--events") {
            //     eprintln!("Events:");
            //     for item in events {
            //         eprintln!("\t{:?}", item);
            //     }
            // }

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
        diagnostic::{Diagnostic, Label},
        files::SimpleFile,
        term::{
            termcolor::{ColorChoice, StandardStream},
            Config,
        },
    };
    use oxc_type_synthesis::{
        Diagnostic as TypeCheckDiagnostic, DiagnosticsContainer, ErrorWarningInfo,
    };

    #[allow(clippy::items_after_statements)]
    pub(super) fn print_diagnostics_container(
        error_handler: DiagnosticsContainer,
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

        for item in error_handler.into_iter().rev() {
            // TODO tidy this up:
            let (diagnostic, info) = match item {
                ErrorWarningInfo::Error(error) => (Diagnostic::error(), error),
                ErrorWarningInfo::Warning(warning) => (Diagnostic::warning(), warning),
                ErrorWarningInfo::Info(info) => (Diagnostic::note(), info),
                ErrorWarningInfo::Data(_) => {
                    continue;
                }
            };

            let diagnostic = checker_diagnostic_to_code_span_diagnostic(diagnostic, info);

            emit(&files, &diagnostic);

            #[cfg(target_arch = "wasm")]
            fn emit<'a, F: codespan_reporting::files::Files<'a>>(
                files: &F,
                diagnostic: &Diagnostic<F::FileId>,
            ) {
                todo!("buffer then print")
            }

            #[cfg(not(target_arch = "wasm"))]
            fn emit<'a, F: codespan_reporting::files::Files<'a>>(
                files: &'a F,
                diagnostic: &Diagnostic<F::FileId>,
            ) {
                let writer = StandardStream::stderr(ColorChoice::Always);

                // TODO lines in diagnostic could be different
                codespan_reporting::term::emit(
                    &mut writer.lock(),
                    &Config::default(),
                    files,
                    diagnostic,
                )
                .unwrap();
            }
        }
    }

    fn checker_diagnostic_to_code_span_diagnostic(
        diagnostic: Diagnostic<()>,
        information: TypeCheckDiagnostic,
        // source_map: &HashMap<EznoSourceId, usize>,
    ) -> Diagnostic<()> {
        match information {
            TypeCheckDiagnostic::Global(message) => diagnostic.with_message(message),
            TypeCheckDiagnostic::Position { reason: message, pos } => {
                diagnostic.with_labels(vec![Label::primary((), pos).with_message(message)])
            }
            TypeCheckDiagnostic::PositionWithAdditionLabels { reason, pos, labels } => {
                let (labels, notes) =
                    labels.into_iter().partition::<Vec<_>, _>(|(_, value)| value.is_some());

                diagnostic
                    .with_labels(
                        iter::once(Label::primary((), pos).with_message(reason))
                            .chain(labels.into_iter().map(|(message, pos)| {
                                let pos = pos.unwrap();
                                Label::secondary((), pos).with_message(message)
                            }))
                            .collect(),
                    )
                    .with_notes(notes.into_iter().map(|(message, _)| message).collect())
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
