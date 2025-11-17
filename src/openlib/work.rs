//! OpenLibrary work schemas.
use friendly::scalar;
use parquet_derive::ParquetRecordWriter;

use crate::arrow::*;
use crate::prelude::*;

use super::key::parse_ol_key;
use super::key::KS_WORK;
pub use super::source::OLWorkRecord;
use super::source::Row;
use super::subject::SubjectEntry;

/// Work row in extracted Parquet.
#[derive(Debug, Clone, ParquetRecordWriter)]
pub struct WorkRec {
    pub id: u32,
    pub key: String,
    pub title: Option<String>,
}

/// Work-author link in extracted Parquet.
#[derive(Debug, Clone, ParquetRecordWriter)]
pub struct WorkAuthorRec {
    pub id: u32,
    pub pos: i16,
    pub author: u32,
}

/// Work-subject record in extracted Parquet.
#[derive(Debug, Clone, ParquetRecordWriter)]
pub struct WorkSubjectRec {
    pub id: u32,
    pub subj_type: u8,
    pub subject: String,
}

impl From<SubjectEntry> for WorkSubjectRec {
    fn from(value: SubjectEntry) -> Self {
        WorkSubjectRec {
            id: value.entity as u32,
            subj_type: value.subj_type.into(),
            subject: value.subject,
        }
    }
}

/// Process author source records into Parquet.
///
/// This must be run **after** the author processor.
pub struct WorkProcessor {
    rec_writer: TableWriter<WorkRec>,
    author_writer: TableWriter<WorkAuthorRec>,
    subject_writer: TableWriter<WorkSubjectRec>,
}

impl WorkProcessor {
    /// Create a new work processor.
    pub fn new() -> Result<WorkProcessor> {
        Ok(WorkProcessor {
            rec_writer: TableWriter::open("works.parquet")?,
            author_writer: TableWriter::open("work-authors.parquet")?,
            subject_writer: TableWriter::open("work-subjects.parquet")?,
        })
    }
}

impl ObjectWriter<Row<OLWorkRecord>> for WorkProcessor {
    fn write_object(&mut self, row: Row<OLWorkRecord>) -> Result<()> {
        let id = parse_ol_key(&row.key, KS_WORK)?;

        self.rec_writer.write_object(WorkRec {
            id,
            key: row.key.clone(),
            title: row.record.title.clone(),
        })?;

        for pos in 0..row.record.authors.len() {
            let akey = row.record.authors[pos].id()?;
            if let Some(aid) = akey {
                let pos = pos as i16;
                self.author_writer.write_object(WorkAuthorRec {
                    id,
                    pos,
                    author: aid,
                })?;
            }
        }

        for sr in row.record.subjects.subject_records(id) {
            self.subject_writer.write_object(sr.into())?;
        }

        Ok(())
    }

    fn finish_objects(self) -> Result<usize> {
        let nr = self.rec_writer.finish_objects()?;
        info!("wrote {} work records", scalar(nr));
        let na = self.author_writer.finish_objects()?;
        info!("wrote {} work-author records", scalar(na));
        let ns = self.subject_writer.finish_objects()?;
        info!("wrote {} work-subject records", scalar(ns));
        Ok(nr)
    }
}
