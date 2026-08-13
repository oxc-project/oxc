use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::Serialize;

use crate::{
    generator::generate,
    minify,
    oracle::{Comparison, Oracle},
};

#[derive(Debug, Clone)]
pub struct CampaignOptions {
    pub start_seed: u64,
    pub iterations: u64,
    pub timeout_ms: u64,
    pub batch_size: usize,
    pub mangle: bool,
}

impl CampaignOptions {
    /// Reject settings that would make the campaign silently test nothing.
    ///
    /// # Errors
    ///
    /// Returns a human-readable message when an option is outside its usable range.
    ///
    /// Node's `vm` timeout must be a positive integer that fits in a `u32`.
    /// Outside that range `runInNewContext` throws `ERR_OUT_OF_RANGE` *before*
    /// evaluating the program, which the oracle cannot distinguish from the
    /// generated program throwing on its own. Every seed would then be skipped
    /// and the campaign would report success without having compared anything.
    pub fn validate(&self) -> Result<(), String> {
        if self.timeout_ms == 0 || self.timeout_ms > u64::from(u32::MAX) {
            return Err(format!(
                "--timeout-ms must be between 1 and {}, got {}",
                u32::MAX,
                self.timeout_ms
            ));
        }
        if self.batch_size == 0 {
            return Err("--batch-size must be at least 1".to_owned());
        }
        // The campaign walks the half-open range `start_seed .. start_seed +
        // iterations`. Letting that end saturate at `u64::MAX` would quietly
        // check fewer seeds than were asked for and still report success.
        if self.iterations > 0 && self.start_seed.checked_add(self.iterations).is_none() {
            return Err(format!(
                "--seed plus --iterations must not exceed {}, got {} + {}",
                u64::MAX,
                self.start_seed,
                self.iterations
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CampaignSummary {
    pub checked: u64,
    pub skipped: u64,
    pub max_minifier_iterations: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct Failure {
    pub seed: u64,
    #[serde(skip)]
    pub original: String,
    #[serde(skip)]
    pub minified: String,
    pub minifier_iterations: u8,
    pub comparison: Comparison,
}

#[derive(Debug)]
pub enum CampaignResult {
    Completed(CampaignSummary),
    Failed { summary: CampaignSummary, failure: Failure },
    HarnessError { seed: u64, message: String },
    MinifierError { seed: u64, source: String, message: String },
}

struct Program {
    seed: u64,
    original: String,
    minified: String,
    minifier_iterations: u8,
}

pub fn run(options: &CampaignOptions) -> CampaignResult {
    let oracle = Oracle::new(options.timeout_ms);
    let batch_size = options.batch_size.max(1);
    let mut summary = CampaignSummary { checked: 0, skipped: 0, max_minifier_iterations: 0 };
    let end_seed = options.start_seed.saturating_add(options.iterations);
    let mut batch_start = options.start_seed;

    while batch_start < end_seed {
        let batch_end = batch_start.saturating_add(batch_size as u64).min(end_seed);
        // Reserve for the seeds this batch actually covers. `batch_size` is
        // user-supplied and can dwarf the requested range, and reserving that
        // instead aborts the process with a capacity overflow.
        let capacity = usize::try_from(batch_end - batch_start).unwrap_or(batch_size);
        let mut programs = Vec::with_capacity(capacity);
        for seed in batch_start..batch_end {
            let original = generate(seed);
            let minified = match minify(&original, options.mangle) {
                Ok(minified) => minified,
                Err(message) => {
                    return CampaignResult::MinifierError { seed, source: original, message };
                }
            };
            programs.push(Program {
                seed,
                original,
                minified: minified.code,
                minifier_iterations: minified.iterations,
            });
        }

        let cases: Vec<_> = programs
            .iter()
            .map(|program| (program.original.as_str(), program.minified.as_str()))
            .collect();
        let comparisons = oracle.compare_many(&cases);
        for (program, comparison) in programs.into_iter().zip(comparisons) {
            match comparison {
                Comparison::Equivalent { .. } => summary.checked += 1,
                Comparison::Skipped { .. } => summary.skipped += 1,
                Comparison::Mismatch { .. } => {
                    return CampaignResult::Failed {
                        summary,
                        failure: Failure {
                            seed: program.seed,
                            original: program.original,
                            minified: program.minified,
                            minifier_iterations: program.minifier_iterations,
                            comparison,
                        },
                    };
                }
                Comparison::HarnessError { message } => {
                    return CampaignResult::HarnessError { seed: program.seed, message };
                }
            }
            summary.max_minifier_iterations =
                summary.max_minifier_iterations.max(program.minifier_iterations);
        }
        batch_start = batch_end;
    }

    CampaignResult::Completed(summary)
}

/// Write the original source, the minified source and a JSON report for a mismatch.
///
/// # Errors
///
/// Returns any I/O error from creating `directory` or writing the artifacts.
pub fn save_failure(failure: &Failure, directory: &Path) -> io::Result<Vec<PathBuf>> {
    fs::create_dir_all(directory)?;
    let stem = format!("seed-{}", failure.seed);
    let original_path = directory.join(format!("{stem}.js"));
    let minified_path = directory.join(format!("{stem}.min.js"));
    let report_path = directory.join(format!("{stem}.json"));

    fs::write(&original_path, &failure.original)?;
    fs::write(&minified_path, &failure.minified)?;
    let report = serde_json::to_vec_pretty(failure).map_err(io::Error::other)?;
    fs::write(&report_path, report)?;
    Ok(vec![original_path, minified_path, report_path])
}

#[cfg(test)]
mod tests {
    use super::{CampaignOptions, CampaignResult, run};

    #[test]
    fn rejects_timeout_outside_node_range() {
        // `0` is the dangerous one: `vm.runInNewContext` throws
        // `ERR_OUT_OF_RANGE` before running the program, the oracle records it
        // as "the original threw", and every seed is skipped.
        for timeout_ms in [0, u64::from(u32::MAX) + 1] {
            let options = CampaignOptions { timeout_ms, ..default_options() };
            assert!(options.validate().is_err(), "expected {timeout_ms} to be rejected");
        }
    }

    #[test]
    fn rejects_seed_range_overflow() {
        // Saturating at `u64::MAX` would silently run fewer seeds than asked for.
        let options =
            CampaignOptions { start_seed: u64::MAX - 1, iterations: 2, ..default_options() };
        assert!(options.validate().is_err());
    }

    #[test]
    fn batch_size_larger_than_the_seed_range_does_not_over_allocate() {
        // Reserving `batch_size` up front aborts with a capacity overflow long
        // before the single requested seed is generated.
        let result =
            run(&CampaignOptions { iterations: 1, batch_size: usize::MAX, ..default_options() });
        assert!(matches!(result, CampaignResult::Completed(summary) if summary.checked == 1));
    }

    #[test]
    fn rejects_zero_batch_size() {
        assert!(CampaignOptions { batch_size: 0, ..default_options() }.validate().is_err());
    }

    #[test]
    fn accepts_usable_options() {
        assert!(default_options().validate().is_ok());
    }

    fn default_options() -> CampaignOptions {
        CampaignOptions {
            start_seed: 0,
            iterations: 10,
            timeout_ms: 100,
            batch_size: 10,
            mangle: false,
        }
    }

    #[test]
    fn small_campaign_completes() {
        let result = run(&default_options());
        assert!(matches!(
            result,
            CampaignResult::Completed(summary) if summary.checked == 10 && summary.skipped == 0
        ));
    }
}
