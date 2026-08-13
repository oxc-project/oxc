#![expect(clippy::print_stdout, clippy::print_stderr)]
use std::{error::Error, path::PathBuf};

use oxc_minifier_fuzz::campaign::{CampaignOptions, CampaignResult, run, save_failure};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = pico_args::Arguments::from_env();
    if args.contains(["-h", "--help"]) {
        print_help();
        return Ok(());
    }

    let mangle = args.contains("--mangle");
    let options = CampaignOptions {
        start_seed: args.opt_value_from_str("--seed")?.unwrap_or(0),
        iterations: args.opt_value_from_str("--iterations")?.unwrap_or(1_000),
        timeout_ms: args.opt_value_from_str("--timeout-ms")?.unwrap_or(100),
        batch_size: args.opt_value_from_str("--batch-size")?.unwrap_or(100),
        mangle,
    };
    let save_dir: PathBuf = args
        .opt_value_from_os_str("--save-dir", |value| Ok::<_, &'static str>(PathBuf::from(value)))?
        .unwrap_or_else(|| PathBuf::from("target/minifier-fuzz"));
    let remaining = args.finish();
    if !remaining.is_empty() {
        return Err(format!("unexpected arguments: {remaining:?}").into());
    }
    options.validate()?;

    match run(&options) {
        CampaignResult::Completed(summary) => {
            println!(
                "checked {} seeds from {}, skipped {}, max minifier iterations {}",
                summary.checked,
                options.start_seed,
                summary.skipped,
                summary.max_minifier_iterations
            );
            // A campaign that compared nothing must not look like a pass. The
            // option validation above covers the known cause, but the generator
            // could also drift into emitting only programs that throw.
            if options.iterations > 0 && summary.checked == 0 {
                return Err(format!(
                    "no seed was compared: all {} seeds were skipped",
                    summary.skipped
                )
                .into());
            }
            Ok(())
        }
        CampaignResult::Failed { summary, failure } => {
            let paths = save_failure(&failure, &save_dir)?;
            eprintln!(
                "semantic mismatch at seed {} after {} checked seeds: {:#?}",
                failure.seed, summary.checked, failure.comparison
            );
            eprintln!("saved failure artifacts:");
            for path in paths {
                eprintln!("  {}", path.display());
            }
            Err("minifier semantic mismatch".into())
        }
        CampaignResult::HarnessError { seed, message } => {
            Err(format!("oracle failed at seed {seed}: {message}").into())
        }
        CampaignResult::MinifierError { seed, source, message } => {
            Err(format!("minifier rejected generated input at seed {seed}: {message}\n{source}")
                .into())
        }
    }
}

fn print_help() {
    println!(
        "oxc_minifier_fuzz\n\n\
         Generate deterministic JavaScript programs, compress them with oxc_minifier,\n\
         and compare observable behavior in isolated Node.js VM contexts.\n\n\
         Options:\n\
           --seed <N>          first seed (default: 0)\n\
           --iterations <N>    number of seeds (default: 1000)\n\
           --timeout-ms <N>    VM timeout per program, 1..=4294967295 (default: 100)\n\
           --batch-size <N>    programs per Node.js process, at least 1 (default: 100)\n\
           --mangle            also mangle names (default: compression only)\n\
           --save-dir <PATH>   mismatch artifacts (default: target/minifier-fuzz)\n"
    );
}
