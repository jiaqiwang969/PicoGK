//! Simple CSV table utilities

use crate::{Error, Result};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub trait DataTable {
    fn max_column_count(&self) -> usize;
    fn column_id(&self, column: usize) -> String;
    fn find_column(&self, name: &str) -> Option<usize>;
    fn row_count(&self) -> usize;
    fn get_at(&self, row: usize, column: usize) -> String;
    fn set_column_ids(&mut self, ids: Vec<String>);
    fn add_row(&mut self, row: Vec<String>);
}

pub struct CsvTable {
    column_ids: Vec<String>,
    rows: Vec<Vec<String>>,
    key_column: usize,
    max_column_count: usize,
}

impl CsvTable {
    pub fn new(column_ids: Option<Vec<String>>) -> Self {
        let ids = column_ids.unwrap_or_default();
        let max_column_count = ids.len();
        Self {
            column_ids: ids,
            rows: Vec::new(),
            key_column: 0,
            max_column_count,
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P, delimiters: &str) -> Result<Self> {
        let file = File::open(path.as_ref())
            .map_err(|e| Error::FileLoad(format!("Failed to open CSV: {}", e)))?;
        let reader = BufReader::new(file);

        let mut column_ids: Option<Vec<String>> = None;
        let mut rows = Vec::new();
        let mut max_column_count = 0usize;

        for line in reader.lines() {
            let line = line.map_err(|e| Error::FileLoad(format!("Failed to read CSV: {}", e)))?;
            if line.trim().is_empty() {
                continue;
            }

            let cols = split_with_delimiters(&line, delimiters);

            if column_ids.is_none() {
                column_ids = Some(cols);
            } else {
                max_column_count = max_column_count.max(cols.len());
                rows.push(cols);
            }
        }

        let column_ids =
            column_ids.ok_or_else(|| Error::FileLoad("No content in CSV file".to_string()))?;
        max_column_count = max_column_count.max(column_ids.len());

        Ok(Self {
            column_ids,
            rows,
            key_column: 0,
            max_column_count,
        })
    }

    pub fn save<P: AsRef<Path>>(&self, path: P, delimiter: char) -> Result<()> {
        let file = File::create(path.as_ref())
            .map_err(|e| Error::FileSave(format!("Failed to save CSV: {}", e)))?;
        let mut writer = BufWriter::new(file);

        write_row(&mut writer, &self.column_ids, delimiter)?;
        for row in &self.rows {
            write_row(&mut writer, row, delimiter)?;
        }

        Ok(())
    }

    pub fn set_key_column(&mut self, column: usize) {
        self.key_column = column;
    }

    pub fn get_by_key_float(&self, key: &str) -> Option<f32> {
        self.get_by_key_string(key)
            .and_then(|s| s.parse::<f32>().ok())
    }

    pub fn get_by_key_string(&self, key: &str) -> Option<String> {
        let mut parts = key.splitn(2, '.');
        let row_name = parts.next()?.trim();
        let column_name = parts.next()?.trim();

        let column = self.find_column(column_name)?;
        for row in &self.rows {
            if row.len() <= self.key_column {
                continue;
            }
            if row[self.key_column].eq_ignore_ascii_case(row_name) {
                if column < row.len() {
                    return Some(row[column].clone());
                }
                return Some(String::new());
            }
        }
        None
    }
}

impl DataTable for CsvTable {
    fn max_column_count(&self) -> usize {
        self.max_column_count
    }

    fn column_id(&self, column: usize) -> String {
        self.column_ids.get(column).cloned().unwrap_or_default()
    }

    fn find_column(&self, name: &str) -> Option<usize> {
        self.column_ids
            .iter()
            .position(|id| id.eq_ignore_ascii_case(name))
    }

    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn get_at(&self, row: usize, column: usize) -> String {
        if row >= self.rows.len() {
            return String::new();
        }
        let cols = &self.rows[row];
        if column >= cols.len() {
            return String::new();
        }
        cols[column].clone()
    }

    fn set_column_ids(&mut self, ids: Vec<String>) {
        self.max_column_count = self.max_column_count.max(ids.len());
        self.column_ids = ids;
    }

    fn add_row(&mut self, row: Vec<String>) {
        self.max_column_count = self.max_column_count.max(row.len());
        self.rows.push(row);
    }
}

fn split_with_delimiters(line: &str, delimiters: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut start = 0usize;

    for (idx, ch) in line.char_indices() {
        if delimiters.contains(ch) {
            result.push(line[start..idx].trim().to_string());
            start = idx + ch.len_utf8();
        }
    }

    if start <= line.len() {
        result.push(line[start..].trim().to_string());
    }

    result
}

fn write_row<W: Write>(writer: &mut W, row: &[String], delimiter: char) -> Result<()> {
    let mut first = true;
    for item in row {
        if !first {
            writer.write_all(&[delimiter as u8])?;
        }
        first = false;
        writer.write_all(item.as_bytes())?;
    }
    writer.write_all(b"\n")?;
    Ok(())
}
