use oxc_coverage::AppArgs;
use pico_args::Arguments;
use rayon::ThreadPoolBuilder;

type TaskRunner = fn(&AppArgs);

struct TaskRegistry {
    tasks: &'static [(&'static str, TaskRunner)],
}

impl TaskRegistry {
    const fn new() -> Self {
        Self {
            tasks: &[
                ("parser", AppArgs::run_parser),
                ("semantic", AppArgs::run_semantic),
                ("codegen", AppArgs::run_codegen),
                // ("formatter", AppArgs::run_formatter),
                ("transformer", AppArgs::run_transformer),
                ("transpiler", AppArgs::run_transpiler),
                ("minifier", AppArgs::run_minifier),
                ("runtime", AppArgs::run_runtime),
                ("estree", AppArgs::run_estree),
            ],
        }
    }

    fn find_task(&self, name: &str) -> Option<TaskRunner> {
        self.tasks.iter().find(|(task_name, _)| *task_name == name).map(|(_, runner)| *runner)
    }

    fn run_task(&self, name: &str, args: &AppArgs) {
        match name {
            "all" => {
                args.run_default();
                args.run_runtime();
            }
            _ => {
                if let Some(runner) = self.find_task(name) {
                    runner(args);
                } else {
                    args.run_default();
                }
            }
        }
    }
}

fn parse_args() -> Result<(Option<String>, AppArgs), String> {
    let mut args = Arguments::from_env();
    let command = args.subcommand().map_err(|e| format!("Failed to parse subcommand: {e}"))?;

    let app_args = AppArgs {
        debug: args.contains("--debug"),
        filter: args.opt_value_from_str("--filter").map_err(|e| format!("Invalid filter: {e}"))?,
        detail: args.contains("--detail"),
        diff: args.contains("--diff"),
    };

    Ok((command, app_args))
}

fn main() {
    let (command, args) = parse_args().unwrap_or_else(|err| {
        eprintln!("Error parsing arguments: {err}");
        std::process::exit(1);
    });

    if args.debug {
        ThreadPoolBuilder::new().num_threads(1).build_global().unwrap();
    }

    let registry = TaskRegistry::new();
    let task = command.as_deref().unwrap_or("default");
    registry.run_task(task, &args);
}
