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
pub fn write_csv<I: Iterator<Item = V> + ExactSizeIterator<Item = V>, V: Serialize>(
    report: I,
    path: &str,
) {
    // If the path ends with ".gz", we use Gzip compression.
    let use_gzip_compression = std::path::Path::new(path)
        .extension()
        .map_or(false, |ext| ext.eq_ignore_ascii_case("gz"));

    if use_gzip_compression {
        let file = std::fs::File::create(path).unwrap();
        let mut writer = csv::Writer::from_writer(flate2::write::GzEncoder::new(
            file,
            flate2::Compression::default(),
        ));

        for record in report {
            writer.serialize(record).unwrap();
        }

        writer.flush().unwrap();
    } else {
        let file = std::fs::File::create(path).unwrap();
        let mut writer = csv::Writer::from_writer(file);

        for record in report {
            writer.serialize(record).unwrap();
        }

        writer.flush().unwrap();
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
        write_csv(report, path);
        return;
    }

    let file = std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .unwrap();
    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(file);

    for record in report {
        writer.serialize(record).unwrap();
    }

    writer.flush().unwrap();
}

/// CSV reader for a given deserializable type.
///
/// # Arguments
/// * `path` - The path to the CSV file.
///
/// # Implementation
/// The function uses csv Reader combined with flate2 to read the CSV file.
pub fn read_csv<V: DeserializeOwned>(path: &str) -> Result<Vec<V>, csv::Error> {
    let use_gzip_compression = std::path::Path::new(path)
        .extension()
        .map_or(false, |ext| ext.eq_ignore_ascii_case("gz"));

    if use_gzip_compression {
        let file = std::fs::File::open(path)?;
        let reader = csv::Reader::from_reader(flate2::read::GzDecoder::new(file));

        reader.into_deserialize().collect()
    } else {
        let file = std::fs::File::open(path)?;
        let reader = csv::Reader::from_reader(file);

        reader.into_deserialize().collect()
    }
}
