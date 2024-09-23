//! Utilities to read and write CSV files.

use serde::de::DeserializeOwned;
use serde::Serialize;

/// CSV writer for a given iterator of serializable values.
///
/// # Arguments
/// * `report` - The iterator of serializable values.
/// * `path` - The path to the CSV file.
///
/// # Implementation
/// The function uses csv Writer combined with flate2 to write the CSV file.
pub fn write_report<I: Iterator<Item = V> + ExactSizeIterator<Item = V>, V: Serialize>(
    report: I,
    path: &str,
) {
    let format = FileType::from_path(path);

    match format {
        FileType::CSV { compression } => {
            if compression {
                let file = std::fs::File::create(path).unwrap();
                let mut writer = csv::WriterBuilder::new().has_headers(false).from_writer(
                    flate2::write::GzEncoder::new(file, flate2::Compression::default()),
                );

                for record in report {
                    writer.serialize(record).unwrap();
                }

                writer.flush().unwrap();
            } else {
                let file = std::fs::File::create(path).unwrap();
                let mut writer = csv::WriterBuilder::new()
                    .has_headers(false)
                    .from_writer(file);

                for record in report {
                    writer.serialize(record).unwrap();
                }

                writer.flush().unwrap();
            }
        }
        FileType::JSON => {
            let file = std::fs::File::create(path).unwrap();
            serde_json::to_writer(file, &report.collect::<Vec<V>>()).unwrap();
        }
    }
}

/// CSV write to appen to a given csv file.
///
/// # Arguments
/// * `report` - The iterator of serializable values.
/// * `path` - The path to the CSV file.
pub fn append_csv<I: Iterator<Item = V> + ExactSizeIterator<Item = V>, V: Serialize>(
    report: I,
    path: &str,
) {
    // If the file does not exist, we create it.
    if !std::path::Path::new(path).exists() {
        write_report(report, path);
        return;
    }

    let file = std::fs::OpenOptions::new().append(true).open(path).unwrap();
    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(file);

    for record in report {
        writer.serialize(record).unwrap();
    }

    writer.flush().unwrap();
}

enum FileType {
    CSV { compression: bool },
    JSON,
}

impl FileType {
    fn from_path(path: &str) -> Self {
        if path.ends_with(".csv.gz") {
            FileType::CSV { compression: true }
        } else if path.ends_with(".csv") {
            FileType::CSV { compression: false }
        } else if path.ends_with(".json") {
            FileType::JSON
        } else {
            panic!("Unsupported file format.");
        }
    }
}

/// CSV reader for a given deserializable type.
///
/// # Arguments
/// * `path` - The path to the CSV file.
///
/// # Implementation
/// The function uses csv Reader combined with flate2 to read the CSV file.
pub fn read_report<V: DeserializeOwned>(path: &str) -> Option<Vec<V>> {
    let format = FileType::from_path(path);

    match format {
        FileType::CSV { compression } => {
            let file = std::fs::File::open(path).ok()?;
            if compression {
                let reader = csv::Reader::from_reader(flate2::read::GzDecoder::new(file));
                reader
                    .into_deserialize()
                    .collect::<Result<Vec<V>, _>>()
                    .ok()
            } else {
                let reader = csv::Reader::from_reader(file);
                reader
                    .into_deserialize()
                    .collect::<Result<Vec<V>, _>>()
                    .ok()
            }
        }
        FileType::JSON => {
            let file = std::fs::File::open(path).ok()?;
            serde_json::from_reader(file).ok()
        }
    }
}
