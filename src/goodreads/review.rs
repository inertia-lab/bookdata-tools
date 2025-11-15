//! GoodReads review data model.
use parquet_derive::ParquetRecordWriter;
pub use serde::Deserialize;

use crate::arrow::*;
use crate::ids::index::IdIndex;
use crate::parsing::dates::*;
use crate::parsing::*;
use crate::prelude::*;

use super::ids::load_id_links;
use super::ids::BookLinkMap;
use super::users::load_user_index;

const OUT_FILE: &'static str = "gr-reviews.parquet";

/// Review records we read from JSON.
#[derive(Deserialize)]
pub struct RawReview {
    pub user_id: String,
    pub book_id: String,
    pub review_id: String,
    pub rating: f32,
    pub review_text: String,
    pub n_votes: i32,
    pub date_added: String,
    pub date_updated: String,
    #[allow(unused)]
    pub read_at: String,
    #[allow(unused)]
    pub started_at: String,
}

/// Review records to write to the Parquet table.
#[derive(ParquetRecordWriter)]
pub struct ReviewRecord {
    /// Internal auto-genereated record identifier.
    pub rec_id: u32,
    /// Review identifier (derived from input).
    pub review_id: i64,
    /// User identifier.
    pub user_id: i32,
    /// GoodReads book identifier.
    pub book_id: i32,
    /// GoodReads work identifier.
    pub work_id: Option<i32>,
    /// Cluster identifier (from [integration clustering][clust]).
    ///
    /// [clust]: https://bookdata.piret.info/data/cluster.html
    pub cluster: i32,
    /// GoodReads “item” identifier
    pub item_id: i32,
    /// Rating associated with this review (if provided).
    pub rating: Option<f32>,
    /// Review text.
    pub review: String,
    /// Number of votes this review has received.
    pub n_votes: i32,
    /// Date review was added.
    pub added: f32,
    /// Date review was updated.
    pub updated: f32,
}

// Object writer to transform and write GoodReads reviews
pub struct ReviewWriter {
    writer: TableWriter<ReviewRecord>,
    users: IdIndex<String>,
    books: BookLinkMap,
    n_recs: u32,
}

impl ReviewWriter {
    // Open a new output
    pub fn open() -> Result<ReviewWriter> {
        let writer = TableWriter::open(OUT_FILE)?;
        let users = load_user_index()?.freeze();
        let books = load_id_links()?;
        Ok(ReviewWriter {
            writer,
            users,
            books,
            n_recs: 0,
        })
    }
}

impl DataSink for ReviewWriter {
    fn output_files(&self) -> Vec<PathBuf> {
        path_list(&[OUT_FILE])
    }
}

impl ObjectWriter<RawReview> for ReviewWriter {
    // Write a single interaction to the output
    fn write_object(&mut self, row: RawReview) -> Result<()> {
        self.n_recs += 1;
        let rec_id = self.n_recs;
        let user_id = self.users.intern_owned(row.user_id)?;
        let book_id: i32 = row.book_id.parse()?;
        let (rev_hi, rev_lo) = decode_hex_i64_pair(&row.review_id)?;
        // review ids were checked for dupluicates in interaction scan, don't repeat here
        let review_id = rev_hi ^ rev_lo;
        let link = self
            .books
            .get(&book_id)
            .ok_or_else(|| anyhow!("unknown book ID"))?;

        self.writer.write_object(ReviewRecord {
            rec_id,
            review_id,
            user_id,
            book_id,
            work_id: link.work_id,
            item_id: link.item_id(),
            cluster: link.cluster,
            review: row.review_text,
            rating: if row.rating > 0.0 {
                Some(row.rating)
            } else {
                None
            },
            n_votes: row.n_votes,
            added: parse_gr_date(&row.date_added).map(check_ts("added", 2000))?,
            updated: parse_gr_date(&row.date_updated).map(check_ts("updated", 2000))?,
        })?;

        Ok(())
    }

    // Clean up and finalize output
    fn finish(self) -> Result<usize> {
        info!(
            "wrote {} records for {} users, closing output",
            self.n_recs,
            self.users.len()
        );
        self.writer.finish()
    }
}
