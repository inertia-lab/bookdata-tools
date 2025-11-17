//! GoodReads book genre records.
use std::collections::HashMap;

use parquet_derive::ParquetRecordWriter;
use serde::Deserialize;

use crate::arrow::*;
use crate::ids::index::IdIndex;
use crate::prelude::*;

const OUT_FILE: &'static str = "gr-book-genres.parquet";
const GENRE_FILE: &'static str = "gr-genres.parquet";

/// Book-genre records as parsed from JSON.
#[derive(Deserialize)]
pub struct RawBookGenre {
    pub book_id: String,
    #[serde(default)]
    pub genres: HashMap<String, i32>,
}

/// Rows in the processed book-genre Parquet table.
#[derive(ParquetRecordWriter)]
pub struct BookGenreRecord {
    pub book_id: i32,
    pub genre_id: i32,
    pub count: i32,
}

/// Object writer to transform and write GoodReads book-genre records
pub struct BookGenreWriter {
    genres: IdIndex<String>,
    writer: TableWriter<BookGenreRecord>,
    n_recs: usize,
}

impl BookGenreWriter {
    /// Open a new output
    pub fn open() -> Result<BookGenreWriter> {
        let writer = TableWriter::open(OUT_FILE)?;
        Ok(BookGenreWriter {
            genres: IdIndex::new(),
            writer,
            n_recs: 0,
        })
    }
}

impl ObjectWriter<RawBookGenre> for BookGenreWriter {
    fn write_object(&mut self, row: RawBookGenre) -> Result<()> {
        let book_id: i32 = row.book_id.parse()?;

        for (genre, count) in row.genres {
            let genre_id = self.genres.intern(&genre)?;
            self.writer.write_object(BookGenreRecord {
                book_id,
                genre_id,
                count,
            })?;

            self.n_recs += 1;
        }

        Ok(())
    }

    fn finish_objects(self) -> Result<usize> {
        self.writer.finish_objects()?;
        self.genres.save(GENRE_FILE, "genre_id", "genre")?;
        Ok(self.n_recs)
    }
}
