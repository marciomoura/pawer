use std::io;
use std::path::Path;

use crate::logger::Logger;

/// Export all logged data to a CSV file.
///
/// The output format is:
/// ```text
/// time,signal_a,signal_b,...
/// 0.000,1.23,4.56,...
/// 0.001,1.24,4.57,...
/// ```
///
/// Signals that were not logged for a given step produce an empty cell.
pub fn export_csv(logger: &Logger, path: &str) -> Result<u64, ExportError> {
    let file = std::fs::File::create(Path::new(path))
        .map_err(|e| ExportError(format!("Failed to create file \"{}\": {}", path, e)))?;

    let mut writer = csv::Writer::from_writer(file);

    let signal_names = logger.signal_names();
    if signal_names.is_empty() {
        return Err(ExportError("No signals to export.".into()));
    }

    // Header row
    let mut header = vec!["time".to_owned()];
    header.extend(signal_names.iter().cloned());
    writer
        .write_record(&header)
        .map_err(|e| ExportError(format!("Failed to write CSV header: {}", e)))?;

    // Data rows
    let records = logger.records();
    for record in records {
        let mut row = vec![format!("{:.6e}", record.time)];
        for name in &signal_names {
            match record.signals.get(name) {
                Some(v) => row.push(format!("{:.6e}", v)),
                None => row.push(String::new()),
            }
        }
        writer
            .write_record(&row)
            .map_err(|e| ExportError(format!("Failed to write CSV row: {}", e)))?;
    }

    writer
        .flush()
        .map_err(|e| ExportError(format!("Failed to flush CSV writer: {}", e)))?;

    Ok(records.len() as u64)
}

#[derive(Debug)]
pub struct ExportError(pub String);

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<io::Error> for ExportError {
    fn from(e: io::Error) -> Self {
        Self(e.to_string())
    }
}
