//! GoodReads book identifier and linking support.
use std::{collections::HashMap, fs::File};

use anyhow::{anyhow, Result};
use arrow::array::{Array, Int32Array};
use log::*;
use parquet::arrow::{arrow_reader::ParquetRecordBatchReaderBuilder, ProjectionMask};
use serde::{Deserialize, Serialize};

use crate::{
    ids::codes::{NS_GR_BOOK, NS_GR_WORK},
    prelude::BDPath,
};

pub type BookLinkMap = HashMap<i32, BookLinkRecord>;

const GR_LINK_FILE: BDPath<'static> = BDPath::new("goodreads/gr-book-link.parquet");

/// Book-link record.
#[derive(Debug, Serialize, Deserialize)]
pub struct BookLinkRecord {
    pub book_id: i32,
    pub work_id: Option<i32>,
    pub cluster: i32,
}

impl BookLinkRecord {
    /// Get the GoodReads item ID for the book (work id, with fallback to book, in numberspace).
    pub fn item_id(&self) -> i32 {
        if let Some(w) = &self.work_id {
            NS_GR_WORK.base() + w
        } else {
            NS_GR_BOOK.base() + self.book_id
        }
    }
}

/// Read a map of book IDs to linking identifiers.
pub fn load_id_links() -> Result<BookLinkMap> {
    let path = GR_LINK_FILE.resolve()?;
    let file = File::open(path)?;

    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let meta = builder.metadata();
    let n_rows = meta.file_metadata().num_rows();
    let project = ProjectionMask::columns(
        meta.file_metadata().schema_descr(),
        ["book_id", "work_id", "cluster"],
    );
    let reader = builder.with_projection(project).build()?;

    let mut map = HashMap::with_capacity(n_rows as usize);

    for batch in reader {
        let batch = batch?;
        assert_eq!(batch.schema().field(0).name(), "book_id");
        assert_eq!(batch.schema().field(1).name(), "work_id");
        assert_eq!(batch.schema().field(2).name(), "cluster");
        let book_id_col = batch
            .column(0)
            .as_any()
            .downcast_ref::<Int32Array>()
            .ok_or_else(|| anyhow!("invalid book_id column"))?;
        let work_id_col = batch
            .column(1)
            .as_any()
            .downcast_ref::<Int32Array>()
            .ok_or_else(|| anyhow!("invalid work_id column"))?;
        let cluster_col = batch
            .column(2)
            .as_any()
            .downcast_ref::<Int32Array>()
            .ok_or_else(|| anyhow!("invalid cluster column"))?;

        for i in 0..batch.num_rows() {
            let rec: BookLinkRecord = BookLinkRecord {
                book_id: book_id_col.value(i),
                work_id: if work_id_col.is_valid(i) {
                    Some(work_id_col.value(i))
                } else {
                    None
                },
                cluster: cluster_col.value(i),
            };
            map.insert(rec.book_id, rec);
        }
    }

    info!("read {} book links from {}", map.len(), GR_LINK_FILE);
    Ok(map)
}
