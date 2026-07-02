//! Sweep polynomial degrees and measure corrected HLL error per precision.
//! Loads cached Monte Carlo reports (P_B.report.json), fits polynomials at each degree,
//! and writes a CSV with columns: precision, degree, uncorrected_error, corrected_error, rate_of_improvement.
#![deny(unsafe_code)]
#![deny(unused_macro_rules)]
#![deny(missing_docs)]

use correction_coefficients::polyfit::{fit_with_degree, FitSample};
use serde::{Deserialize, Serialize};
use std::fs;

/// One row of output CSV.
#[derive(Debug, Serialize)]
struct SweepRow {
    precision: u8,
    bits: u8,
    degree: usize,
    uncorrected_error: String,
    corrected_error: String,
    rate_of_improvement: String,
}

/// Subset of the report schema we need (mirrors test_utils::CardinalitySample).
#[derive(Debug, Clone, Deserialize)]
struct Sample {
    exact_cardinality_mean: f64,
    estimated_cardinality_mean: f64,
    absolute_relative_error_mean: f64,
}

#[derive(Debug, Deserialize)]
struct Report {
    hyperloglog: Vec<Sample>,
}

fn fit_samples(samples: &[Sample]) -> Vec<FitSample> {
    samples
        .iter()
        .filter(|s| s.exact_cardinality_mean > 0.0)
        .map(|s| FitSample {
            raw: s.estimated_cardinality_mean,
            exact: s.exact_cardinality_mean,
            count: 1.0,
        })
        .collect()
}

fn main() {
    let precisions: Vec<u8> = (4..=12).collect();
    let bits: u8 = 6;
    let degrees: Vec<usize> = (1..=12).collect();

    let mut rows: Vec<SweepRow> = Vec::new();

    for &p in &precisions {
        let report_path = format!("{}_{}.report.json", p, bits);
        let report: Report = match fs::read_to_string(&report_path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|e| {
                eprintln!("Failed to parse {}: {}", report_path, e);
                std::process::exit(1);
            }),
            Err(e) => {
                eprintln!("Report not found {}: {}", report_path, e);
                std::process::exit(1);
            }
        };

        let m = (1u64 << p) as f64;

        // Filter to the register-regime domain (below 7.5 * 2^P).
        let filtered: Vec<Sample> = report
            .hyperloglog
            .iter()
            .filter(|s| s.exact_cardinality_mean < 7.5 * m)
            .cloned()
            .collect();

        // Also keep the full set for uncorrected error measurement (all HLL samples).
        let all_samples = &report.hyperloglog;

        // Uncorrected error is the same regardless of degree.
        let uncorrected_error: f64 = all_samples
            .iter()
            .map(|s| s.absolute_relative_error_mean)
            .sum::<f64>()
            / all_samples.len() as f64;

        let fit_data = fit_samples(&filtered);

        for &degree in &degrees {
            let poly = fit_with_degree(&fit_data, m, degree);

            // Measure corrected error over ALL HLL samples (not just the fit domain).
            // Samples outside the fit domain use the raw estimate (clamping in corrected() handles this).
            let corrected_error: f64 = all_samples
                .iter()
                .map(|s| {
                    let corrected = poly.corrected(s.estimated_cardinality_mean, m);
                    (s.exact_cardinality_mean - corrected).abs() / s.exact_cardinality_mean.max(1.0)
                })
                .sum::<f64>()
                / all_samples.len() as f64;

            let rate = uncorrected_error / corrected_error;

            rows.push(SweepRow {
                precision: p,
                bits,
                degree,
                uncorrected_error: format!("{:.6}", uncorrected_error),
                corrected_error: format!("{:.6}", corrected_error),
                rate_of_improvement: format!("{:.4}", rate),
            });
        }

        eprintln!(
            "Precision {}: {} HLL samples, {} in fit domain, uncorrected_err={:.6}",
            p,
            all_samples.len(),
            filtered.len(),
            uncorrected_error
        );
    }

    // Write CSV.
    let output_path = "degree_sweep.csv";
    let header = "precision,bits,degree,uncorrected_error,corrected_error,rate_of_improvement";
    let mut csv_lines = vec![header.to_string()];
    for row in &rows {
        csv_lines.push(format!(
            "{},{},{},{},{},{}",
            row.precision, row.bits, row.degree,
            row.uncorrected_error, row.corrected_error, row.rate_of_improvement
        ));
    }

    fs::write(output_path, csv_lines.join("\n") + "\n").unwrap();
    eprintln!("Wrote {} rows to {}", rows.len(), output_path);
}
