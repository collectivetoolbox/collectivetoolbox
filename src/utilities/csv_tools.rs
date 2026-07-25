/// Tools for loading and querying CSV datasets.
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use anyhow::{Context, Result, anyhow};

/// Options controlling how a CSV is parsed.
#[derive(Debug, Clone, Copy)]
pub struct CsvParseOptions {
    /// Treat the first row as a header.
    pub has_header: bool,
    /// Delimiter byte (defaults to `b','`).
    pub delimiter: u8,
    /*
    /// Whether to use RFC-4180-style quoting.
    ///
    /// Some datasets in this repo are "CSV-ish" and should treat `"` as a
    /// literal character instead of a quoting delimiter.
    pub quoting: bool,*/
}

impl Default for CsvParseOptions {
    fn default() -> Self {
        Self {
            has_header: false,
            delimiter: b',',
        }
    }
}

/// A parsed CSV dataset held fully in memory.
#[derive(Debug, Clone)]
pub struct CsvTable {
    header: Option<Vec<String>>,
    rows: Vec<Vec<String>>,
}

impl CsvTable {
    /// Return the header row, if present.
    pub fn header(&self) -> Option<&[String]> {
        self.header.as_deref()
    }

    /// Return the number of data rows.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Return the maximum number of columns across all rows.
    pub fn max_column_count(&self) -> usize {
        self.rows.iter().map(std::vec::Vec::len).max().unwrap_or(0)
    }

    /// Get row `row_idx`.
    pub fn row(&self, row_idx: usize) -> Option<&[String]> {
        self.rows.get(row_idx).map(std::vec::Vec::as_slice)
    }

    /// Get cell at (row, col).
    pub fn cell(&self, row_idx: usize, col_idx: usize) -> Option<&str> {
        self.rows
            .get(row_idx)
            .and_then(|r| r.get(col_idx))
            .map(std::string::String::as_str)
    }

    /// Get the value for `row_idx` in the column whose header equals
    /// `header_name`.
    pub fn cell_by_header(
        &self,
        row_idx: usize,
        header_name: &str,
    ) -> Option<&str> {
        let col_idx = self.header_index(header_name)?;
        self.cell(row_idx, col_idx)
    }

    /// Get the index of a header column.
    pub fn header_index(&self, header_name: &str) -> Option<usize> {
        self.header.as_ref()?.iter().position(|h| h == header_name)
    }

    /// Get column `col_idx` across all rows.
    ///
    /// Missing cells are returned as `None`.
    pub fn column(&self, col_idx: usize) -> Vec<Option<&str>> {
        self.rows
            .iter()
            .map(|r| r.get(col_idx).map(std::string::String::as_str))
            .collect()
    }

    /// Find the first row index where column `match_col_idx` equals
    /// `match_value`.
    pub fn row_index_where_col_eq(
        &self,
        match_col_idx: usize,
        match_value: &str,
    ) -> Option<usize> {
        self.rows.iter().position(|row| {
            row.get(match_col_idx)
                .is_some_and(|v| v.as_str() == match_value)
        })
    }

    /// Find the first row index where the column named `match_header` equals
    /// `match_value`.
    pub fn row_index_where_header_eq(
        &self,
        match_header: &str,
        match_value: &str,
    ) -> Option<usize> {
        let match_col_idx = self.header_index(match_header)?;
        self.row_index_where_col_eq(match_col_idx, match_value)
    }

    /// Get the value of column `target_col_idx` in the first row where column
    /// `match_col_idx` equals `match_value`.
    pub fn cell_where_col_eq(
        &self,
        target_col_idx: usize,
        match_col_idx: usize,
        match_value: &str,
    ) -> Option<&str> {
        let row_idx =
            self.row_index_where_col_eq(match_col_idx, match_value)?;
        self.cell(row_idx, target_col_idx)
    }

    /// Get the value in the column named `target_header` in the first row where
    /// the column named `match_header` equals `match_value`.
    pub fn cell_where_header_eq(
        &self,
        target_header: &str,
        match_header: &str,
        match_value: &str,
    ) -> Option<&str> {
        let target_col_idx = self.header_index(target_header)?;
        let match_col_idx = self.header_index(match_header)?;
        self.cell_where_col_eq(target_col_idx, match_col_idx, match_value)
    }

    pub fn rows_iter(&self) -> impl Iterator<Item = &[String]> {
        self.rows.iter().map(std::vec::Vec::as_slice)
    }
}

/// Parse CSV data from any `Read`er into memory.
pub fn parse_csv_reader(
    data: &Vec<u8>,
    options: CsvParseOptions,
) -> Result<CsvTable> {
    let reader = std::io::Cursor::new(data);
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(options.has_header)
        .delimiter(options.delimiter)
        // .quoting(options.quoting)
        .from_reader(reader);

    let header = if options.has_header {
        Some(
            rdr.headers()
                .context("reading csv headers")?
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
        )
    } else {
        None
    };

    let mut rows = Vec::new();
    for record in rdr.records() {
        let record = record.context("parsing csv record")?;
        rows.push(
            record
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
        );
    }

    Ok(CsvTable { header, rows })
}

static CSV_TABLE_CACHE: OnceLock<Mutex<HashMap<&'static str, Arc<CsvTable>>>> =
    OnceLock::new();

fn lock_csv_cache<'a>(
    cache: &'a Mutex<HashMap<&'static str, Arc<CsvTable>>>,
) -> Result<std::sync::MutexGuard<'a, HashMap<&'static str, Arc<CsvTable>>>> {
    cache
        .lock()
        .map_err(|_poisoned| anyhow!("csv cache mutex is poisoned"))
}

/// Parse a CSV dataset and cache it by a caller-provided key.
///
/// The key is intended to be scoped by the calling crate (e.g.
/// `"ctb_formats_html::data/netscape-entities-1999.csv"`) so multiple crates
/// can cache datasets without collisions.
pub fn get_or_load_cached(
    cache_key: &'static str,
    loader: impl FnOnce() -> Result<CsvTable>,
) -> Result<Arc<CsvTable>> {
    let cache = CSV_TABLE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

    {
        let guard = lock_csv_cache(cache)?;
        if let Some(existing) = guard.get(cache_key) {
            return Ok(existing.clone());
        }
    }

    let loaded_table = Arc::new(
        loader().with_context(|| format!("loading csv dataset {cache_key}"))?,
    );

    let mut guard = lock_csv_cache(cache)?;
    let entry = guard
        .entry(cache_key)
        .or_insert_with(|| loaded_table.clone());
    Ok(entry.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::atomic::{AtomicUsize, Ordering};

    #[crate::ctb_test]
    fn parse_and_query_with_headers() -> Result<()> {
        let data = "Name,Age\nAlice,30\nBob,40\n";
        let table = parse_csv_reader(
            &data.as_bytes().to_vec(),
            CsvParseOptions {
                has_header: true,
                ..Default::default()
            },
        )?;

        assert_eq!(table.header_index("Name"), Some(0));
        assert_eq!(table.header_index("Age"), Some(1));

        assert_eq!(table.row_count(), 2);
        assert_eq!(table.cell(0, 0), Some("Alice"));
        assert_eq!(table.cell(0, 1), Some("30"));
        assert_eq!(table.cell_by_header(1, "Name"), Some("Bob"));
        assert_eq!(table.cell_by_header(1, "Age"), Some("40"));

        assert_eq!(table.row_index_where_col_eq(0, "Bob"), Some(1));
        assert_eq!(table.row_index_where_header_eq("Name", "Alice"), Some(0));
        assert_eq!(table.cell_where_col_eq(1, 0, "Alice"), Some("30"));
        assert_eq!(
            table.cell_where_header_eq("Age", "Name", "Bob"),
            Some("40")
        );

        let col0 = table.column(0);
        assert_eq!(col0, vec![Some("Alice"), Some("Bob")]);

        Ok(())
    }

    #[crate::ctb_test]
    fn parse_without_headers() -> Result<()> {
        let data = "a,b\nc,d\n";
        let table = parse_csv_reader(
            &data.as_bytes().to_vec(),
            CsvParseOptions {
                has_header: false,
                ..Default::default()
            },
        )?;
        assert_eq!(table.header(), None);
        assert_eq!(table.cell(1, 0), Some("c"));
        Ok(())
    }

    #[crate::ctb_test]
    fn cached_loader_runs_once() -> Result<()> {
        static CALLS: AtomicUsize = AtomicUsize::new(0);

        let first =
            get_or_load_cached("ctb_utilities::tests::example", || {
                CALLS.fetch_add(1, Ordering::SeqCst);
                parse_csv_reader(
                    &"x\n".as_bytes().to_vec(),
                    CsvParseOptions {
                        has_header: false,
                        ..Default::default()
                    },
                )
            })?;

        let second =
            get_or_load_cached("ctb_utilities::tests::example", || {
                CALLS.fetch_add(1, Ordering::SeqCst);
                parse_csv_reader(
                    &"y\n".as_bytes().to_vec(),
                    CsvParseOptions {
                        has_header: false,
                        ..Default::default()
                    },
                )
            })?;

        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(CALLS.load(Ordering::SeqCst), 1);

        Ok(())
    }
}
