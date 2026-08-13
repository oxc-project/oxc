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
        let mut programs = Vec::with_capacity((batch_end - batch_start) as usize);
        for seed in batch_start..batch_end {
            let original = generate(seed);
            let minified = match minify(&original) {
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
    fn small_campaign_completes() {
        let result = run(&CampaignOptions {
            start_seed: 0,
            iterations: 10,
            timeout_ms: 100,
            batch_size: 10,
        });
        assert!(matches!(
            result,
            CampaignResult::Completed(summary) if summary.checked == 10 && summary.skipped == 0
        ));
    }
}
