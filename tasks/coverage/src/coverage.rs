use std::io::Write;

use console::Style;
use similar::{ChangeTag, TextDiff};

use crate::{AppArgs, suite::Case};

/// Coverage report data structure
pub struct CoverageReport<'a, T> {
    pub failed_positives: Vec<&'a T>,
    pub failed_negatives: Vec<&'a T>,
    pub parsed_positives: usize,
    pub passed_positives: usize,
    pub passed_negatives: usize,
    pub all_positives: usize,
    pub all_negatives: usize,
}

impl<'a, T: Case> CoverageReport<'a, T> {
    /// Generate coverage report from test cases
    pub fn from_test_cases(tests: &'a [T]) -> Self {
        let (negatives, positives): (Vec<_>, Vec<_>) =
            tests.iter().partition(|case| case.should_fail());

        let all_positives = positives.len();
        let parsed_positives = positives.iter().filter(|case| case.test_parsed()).count();

        let mut failed_positives =
            positives.into_iter().filter(|case| !case.test_passed()).collect::<Vec<_>>();
        failed_positives.sort_by_key(|case| case.path());

        let passed_positives = all_positives - failed_positives.len();

        let all_negatives = negatives.len();
        let mut failed_negatives =
            negatives.into_iter().filter(|case| !case.test_passed()).collect::<Vec<_>>();
        failed_negatives.sort_by_key(|case| case.path());

        let passed_negatives = all_negatives - failed_negatives.len();

        CoverageReport {
            failed_positives,
            failed_negatives,
            parsed_positives,
            passed_positives,
            passed_negatives,
            all_positives,
            all_negatives,
        }
    }

    /// Print coverage report to writer
    /// # Errors
    #[expect(clippy::cast_precision_loss)]
    pub fn print<W: Write>(
        &self,
        name: &str,
        args: &AppArgs,
        writer: &mut W,
    ) -> std::io::Result<()> {
        let parsed_diff = (self.parsed_positives as f64 / self.all_positives as f64) * 100.0;
        let positive_diff = (self.passed_positives as f64 / self.all_positives as f64) * 100.0;
        let negative_diff = (self.passed_negatives as f64 / self.all_negatives as f64) * 100.0;
        
        writer.write_all(format!("{name} Summary:\n").as_bytes())?;
        let msg = format!(
            "AST Parsed     : {}/{} ({:.2}%)\n",
            self.parsed_positives, self.all_positives, parsed_diff
        );
        writer.write_all(msg.as_bytes())?;
        
        let msg = format!(
            "Positive Passed: {}/{} ({:.2}%)\n",
            self.passed_positives, self.all_positives, positive_diff
        );
        writer.write_all(msg.as_bytes())?;
        
        if self.all_negatives > 0 {
            let msg = format!(
                "Negative Passed: {}/{} ({:.2}%)\n",
                self.passed_negatives, self.all_negatives, negative_diff
            );
            writer.write_all(msg.as_bytes())?;
        }

        if args.should_print_detail() {
            for case in &self.failed_negatives {
                case.print(args, writer)?;
            }
            for case in &self.failed_positives {
                case.print(args, writer)?;
            }
        }
        writer.flush()?;
        Ok(())
    }

    /// Print diff between two strings
    pub fn print_diff<W: Write>(
        writer: &mut W,
        origin_string: &str,
        expected_string: &str,
    ) -> std::io::Result<()> {
        let diff = TextDiff::from_lines(expected_string, origin_string);
        for change in diff.iter_all_changes() {
            let (sign, style) = match change.tag() {
                ChangeTag::Delete => ("-", Style::new().red()),
                ChangeTag::Insert => ("+", Style::new().green()),
                ChangeTag::Equal => continue, // (" ", Style::new()),
            };
            writer.write_all(
                format!("{}{}", style.apply_to(sign).bold(), style.apply_to(change)).as_bytes(),
            )?;
        }
        Ok(())
    }
}