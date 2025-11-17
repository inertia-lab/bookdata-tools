//! GoodReads work schemas and record processing.
use parquet_derive::ParquetRecordWriter;
use serde::Deserialize;

use crate::arrow::*;
use crate::parsing::*;
use crate::prelude::*;

const OUT_FILE: &'static str = "gr-work-info.parquet";

/// Work records as parsed from JSON.
#[derive(Deserialize)]
pub struct RawWork {
    pub work_id: String,
    #[serde(default)]
    pub original_title: String,
    #[serde(default)]
    pub original_publication_year: String,
    #[serde(default)]
    pub original_publication_month: String,
    #[serde(default)]
    #[allow(unused)]
    pub original_publication_day: String,
}

/// Rows in the processed work Parquet table.
#[derive(ParquetRecordWriter)]
pub struct WorkRecord {
    pub work_id: i32,
    pub title: Option<String>,
    pub pub_year: Option<i16>,
    pub pub_month: Option<u8>,
}

/// Object writer to transform and write GoodReads works
pub struct WorkWriter {
    writer: TableWriter<WorkRecord>,
    n_recs: usize,
}

impl WorkWriter {
    /// Open a new output
    pub fn open() -> Result<WorkWriter> {
        let writer = TableWriter::open(OUT_FILE)?;
        Ok(WorkWriter { writer, n_recs: 0 })
    }
}

impl ObjectWriter<RawWork> for WorkWriter {
    fn write_object(&mut self, row: RawWork) -> Result<()> {
        let work_id: i32 = row.work_id.parse()?;

        let pub_year = parse_opt(&row.original_publication_year)?;
        let pub_month = parse_opt(&row.original_publication_month)?;

        self.writer.write_object(WorkRecord {
            work_id,
            title: trim_owned(&row.original_title),
            pub_year,
            pub_month,
        })?;
        self.n_recs += 1;
        Ok(())
    }

    fn finish_objects(self) -> Result<usize> {
        self.writer.finish_objects()?;
        Ok(self.n_recs)
    }
}
