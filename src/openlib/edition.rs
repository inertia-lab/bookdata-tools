//! OpenLibrary edition schemas.
use friendly::scalar;
use parquet_derive::ParquetRecordWriter;

use crate::arrow::*;
use crate::cleaning::isbns::clean_asin_chars;
use crate::cleaning::isbns::clean_isbn_chars;
use crate::prelude::*;

use super::key::parse_ol_key;
use super::key::KS_WORK;
pub use super::source::OLEditionRecord;
use super::source::Row;
use super::subject::SubjectEntry;

/// An edition row in the extracted Parquet.
#[derive(ParquetRecordWriter)]
pub struct EditionRec {
    pub id: u32,
    pub key: String,
    pub title: Option<String>,
}

/// Link between edition and work.
#[derive(ParquetRecordWriter)]
pub struct LinkRec {
    pub edition: u32,
    pub work: u32,
}

/// Edition ISBN record.
#[derive(ParquetRecordWriter)]
pub struct ISBNrec {
    pub edition: u32,
    pub isbn: String,
}

/// Edition author record.
#[derive(ParquetRecordWriter)]
pub struct EditionAuthorRec {
    pub edition: u32,
    pub pos: i16,
    pub author: u32,
}

/// Edition-subject record in extracted Parquet.
#[derive(ParquetRecordWriter)]
pub struct EditionSubjectRec {
    pub id: u32,
    pub subj_type: u8,
    pub subject: String,
}

impl From<SubjectEntry> for EditionSubjectRec {
    fn from(value: SubjectEntry) -> Self {
        EditionSubjectRec {
            id: value.entity,
            subj_type: value.subj_type.into(),
            subject: value.subject,
        }
    }
}

/// Process edition records into Parquet.
///
/// This must be run **after** the author and work processors.
pub struct EditionProcessor {
    last_id: u32,
    rec_writer: TableWriter<EditionRec>,
    link_writer: TableWriter<LinkRec>,
    isbn_writer: TableWriter<ISBNrec>,
    author_writer: TableWriter<EditionAuthorRec>,
    subject_writer: TableWriter<EditionSubjectRec>,
}

impl EditionProcessor {
    pub fn new() -> Result<EditionProcessor> {
        Ok(EditionProcessor {
            last_id: 0,
            rec_writer: TableWriter::open("editions.parquet")?,
            link_writer: TableWriter::open("edition-works.parquet")?,
            isbn_writer: TableWriter::open("edition-isbns.parquet")?,
            author_writer: TableWriter::open("edition-authors.parquet")?,
            subject_writer: TableWriter::open("edition-subjects.parquet")?,
        })
    }

    fn save_isbns(
        &mut self,
        edition: u32,
        isbns: Vec<String>,
        clean: fn(&str) -> String,
    ) -> Result<()> {
        for isbn in isbns {
            let isbn = clean(&isbn);
            // filter but with a reasonable threshold of error
            if isbn.len() >= 8 {
                self.isbn_writer.write_object(ISBNrec { edition, isbn })?;
            }
        }

        Ok(())
    }
}

impl ObjectWriter<Row<OLEditionRecord>> for EditionProcessor {
    fn write_object(&mut self, row: Row<OLEditionRecord>) -> Result<()> {
        self.last_id += 1;
        let id = self.last_id;

        self.rec_writer.write_object(EditionRec {
            id,
            key: row.key.clone(),
            title: row.record.title.clone(),
        })?;

        self.save_isbns(id, row.record.isbn_10, clean_isbn_chars)?;
        self.save_isbns(id, row.record.isbn_13, clean_isbn_chars)?;
        self.save_isbns(id, row.record.asin, clean_asin_chars)?;

        for work in row.record.works {
            let key = work.key;
            let work = parse_ol_key(&key, KS_WORK)?;
            self.link_writer
                .write_object(LinkRec { edition: id, work })?;
        }

        for pos in 0..row.record.authors.len() {
            let author = row.record.authors[pos].id()?;
            if let Some(aid) = author {
                let pos = pos as i16;
                self.author_writer.write_object(EditionAuthorRec {
                    edition: id,
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
        let n = self.rec_writer.finish_objects()?;
        info!("wrote {} edition records", scalar(n));
        let n = self.author_writer.finish_objects()?;
        info!("wrote {} edition-author records", scalar(n));
        let n = self.link_writer.finish_objects()?;
        info!("wrote {} edition-work records", scalar(n));
        let n = self.isbn_writer.finish_objects()?;
        info!("wrote {} edition-isbn records", scalar(n));
        let n = self.subject_writer.finish_objects()?;
        info!("wrote {} edition-subject records", scalar(n));
        Ok(self.last_id as usize)
    }
}

impl EditionProcessor {}
