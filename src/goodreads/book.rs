//! GoodReads book schemas and record processing.
use parquet_derive::ParquetRecordWriter;
use serde::Deserialize;

use crate::arrow::*;
use crate::cleaning::isbns::*;
use crate::ids::codes::NS_GR_BOOK;
use crate::ids::codes::NS_GR_WORK;
use crate::parsing::*;
use crate::prelude::*;

const ID_FILE: &'static str = "gr-book-ids.parquet";
const INFO_FILE: &'static str = "gr-book-info.parquet";
const SERIES_FILE: &'static str = "gr-book-series.parquet";
const AUTHOR_FILE: &'static str = "gr-book-authors.parquet";

/// The raw records we read from JSON
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct RawBook {
    pub book_id: String,
    pub work_id: String,
    pub isbn: String,
    pub isbn13: String,
    pub asin: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub authors: Vec<RawAuthor>,
    #[serde(default)]
    pub publication_year: String,
    #[serde(default)]
    pub publication_month: String,
    #[serde(default)]
    pub publication_day: String,
    #[serde(default)]
    pub series: Vec<String>,
}

/// The raw author records from JSON.
#[derive(Deserialize)]
pub struct RawAuthor {
    pub author_id: String,
    #[serde(default)]
    pub role: String,
}

/// the book ID records to write to Parquet.
#[derive(ParquetRecordWriter)]
pub struct BookIdRecord {
    /// The book ID, converted from UUID.
    pub book_id: i32,
    /// The work ID, converted from UUID.
    pub work_id: Option<i32>,
    /// The integrated item ID, converted from book and work IDs projected into number spaces.
    pub item_id: i32,
    pub isbn10: Option<String>,
    pub isbn13: Option<String>,
    pub asin: Option<String>,
}

/// book info records to actually write
#[derive(ParquetRecordWriter)]
pub struct BookRecord {
    pub book_id: i32,
    pub title: Option<String>,
    pub pub_year: Option<u16>,
    pub pub_month: Option<u8>,
}

/// book series linking records
#[derive(ParquetRecordWriter)]
pub struct BookSeriesRecord {
    pub book_id: i32,
    pub series: String,
}

/// book author linking records
#[derive(ParquetRecordWriter)]
pub struct BookAuthorRecord {
    pub book_id: i32,
    pub author_id: i32,
    pub position: i16,
    pub role: Option<String>,
}

/// Output handler for GoodReads books.
pub struct BookWriter {
    id_out: TableWriter<BookIdRecord>,
    info_out: TableWriter<BookRecord>,
    author_out: TableWriter<BookAuthorRecord>,
    series_out: TableWriter<BookSeriesRecord>,
}

impl BookWriter {
    pub fn open() -> Result<BookWriter> {
        let id_out = TableWriter::open(ID_FILE)?;
        let info_out = TableWriter::open(INFO_FILE)?;
        let author_out = TableWriter::open(AUTHOR_FILE)?;
        let series_out = TableWriter::open(SERIES_FILE)?;
        Ok(BookWriter {
            id_out,
            info_out,
            author_out,
            series_out,
        })
    }
}

impl ObjectWriter<RawBook> for BookWriter {
    fn write_object(&mut self, row: RawBook) -> Result<()> {
        let book_id = row.book_id.parse()?;
        let work_id = parse_opt(&row.work_id)?;
        let item_id = if let Some(w) = work_id {
            NS_GR_WORK.to_code(w)
        } else {
            NS_GR_BOOK.to_code(book_id)
        };

        self.id_out.write_object(BookIdRecord {
            book_id,
            work_id,
            item_id,
            isbn10: trim_opt(&row.isbn)
                .map(|s| clean_asin_chars(s))
                .filter(|s| s.len() >= 7),
            isbn13: trim_opt(&row.isbn13)
                .map(|s| clean_asin_chars(s))
                .filter(|s| s.len() >= 7),
            asin: trim_opt(&row.asin)
                .map(|s| clean_asin_chars(s))
                .filter(|s| s.len() >= 7),
        })?;

        let pub_year = parse_opt(&row.publication_year)?;
        let pub_month = parse_opt(&row.publication_month)?;

        self.info_out.write_object(BookRecord {
            book_id,
            title: trim_owned(&row.title),
            pub_year,
            pub_month,
        })?;

        for (i, author) in row.authors.into_iter().enumerate() {
            self.author_out.write_object(BookAuthorRecord {
                book_id,
                author_id: author.author_id.parse()?,
                position: (i as i16) + 1,
                role: Some(author.role).filter(|s| !s.is_empty()),
            })?;
        }

        for series in row.series {
            self.series_out
                .write_object(BookSeriesRecord { book_id, series })?;
        }

        Ok(())
    }

    fn finish_objects(self) -> Result<usize> {
        self.id_out.finish_objects()?;
        self.info_out.finish_objects()?;
        self.author_out.finish_objects()?;
        self.series_out.finish_objects()?;
        Ok(0)
    }
}
