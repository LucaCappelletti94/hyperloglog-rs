use crate::estimation_tests::Header;
use indicatif::ProgressIterator;

pub fn standard_deviation(values: &[f64], mean: f64) -> f64 {
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    variance.sqrt()
}

pub fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

pub fn mean_usize(values: &[usize]) -> f64 {
    values.iter().sum::<usize>() as f64 / values.len() as f64
}


pub(crate) fn write_csv<I: Iterator<Item = V> + ExactSizeIterator<Item = V>, V: Header + Into<Vec<String>>>(
    report: I,
    path: &str,
) {
    let mut writer = csv::Writer::from_path(&path).unwrap();

    writer.write_record(V::header()).unwrap();

    let progress_bar = indicatif::ProgressBar::new(report.len() as u64);

    progress_bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("Writing CSV: [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}")
            .unwrap()
            .progress_chars("##-"),
    );

    for row in report.progress_with(progress_bar) {
        let row: Vec<String> = row.into();
        writer.write_record(row).unwrap();
    }

    writer.flush().unwrap();
}