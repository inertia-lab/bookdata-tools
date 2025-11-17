//! GoodReads work schemas and record processing.
use parquet_derive::ParquetRecordWriter;
use serde::Deserialize;

use crate::arrow::*;
use crate::parsing::*;
use crate::prelude::*;

const OUT_FILE: &'static str = "gr-author-info.parquet";

/// Author records as parsed from JSON.
#[derive(Deserialize)]
pub struct RawAuthor {
    pub author_id: String,
    pub name: String,
}

/// Rows in the processed work Parquet table.
#[derive(ParquetRecordWriter)]
pub struct AuthorRecord {
    pub author_id: i32,
    pub name: Option<String>,
}

/// Object writer to transform and write GoodReads works
pub struct AuthorWriter {
    writer: TableWriter<AuthorRecord>,
    n_recs: usize,
}

impl AuthorWriter {
    /// Open a new output
    pub fn open() -> Result<AuthorWriter> {
        let writer = TableWriter::open(OUT_FILE)?;
        Ok(AuthorWriter { writer, n_recs: 0 })
    }
}

impl ObjectWriter<RawAuthor> for AuthorWriter {
    fn write_object(&mut self, row: RawAuthor) -> Result<()> {
        let author_id: i32 = row.author_id.parse()?;

        self.writer.write_object(AuthorRecord {
            author_id,
            name: trim_owned(&row.name),
        })?;

        self.n_recs += 1;
        Ok(())
    }

    fn finish_objects(self) -> Result<usize> {
        self.writer.finish_objects()?;
        Ok(self.n_recs)
    }
}
