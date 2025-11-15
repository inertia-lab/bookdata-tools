//! Summarize author gender information for clusters.
//!
//! This script reads the cluster author information and author gender
//! information, in order to aggregate author genders for each cluster.
//!
//! We use a lot of left joins so that we can compute statistics across
//! the integration pipeline.
use std::path::{Path, PathBuf};

use parquet_derive::ParquetRecordWriter;
use serde::{Deserialize, Serialize};

use crate::arrow::*;
use crate::ids::codes::*;
use crate::prelude::*;

mod authors;
mod clusters;

// #[derive(Display, FromStr, Debug)]
// #[display(style="lowercase")]
// enum GenderSource {
//   VIAF,
// }

#[derive(Args, Debug)]
#[command(name = "extract-author-genders")]
/// Extract cluster author gender data from extracted book data.
pub struct AuthorGender {
    /// Specify output file
    #[arg(short = 'o', long = "output")]
    output: PathBuf,

    /// Specify the cluster-author file.
    #[arg(short = 'A', long = "cluster-authors")]
    author_file: PathBuf,
}

/// Record format for saving gender information.
#[derive(Serialize, Deserialize, Clone, ParquetRecordWriter)]
struct ClusterGenderInfo {
    cluster: i32,
    gender: String,
}

fn save_genders(clusters: Vec<i32>, genders: clusters::ClusterTable, outf: &Path) -> Result<()> {
    info!("writing cluster genders to {}", outf.display());
    let mut out = TableWriter::open(outf)?;

    for cluster in clusters {
        let mut gender = "no-book-author".to_owned();
        if NS_ISBN.from_code(cluster).is_some() {
            gender = "no-book".to_owned();
        }
        if let Some(stats) = genders.get(&cluster) {
            if stats.n_book_authors == 0 {
                assert!(stats.genders.is_empty());
                gender = "no-book-author".to_owned() // shouldn't happen but 🤷‍♀️
            } else if stats.n_author_recs == 0 {
                assert!(stats.genders.is_empty());
                gender = "no-author-rec".to_owned()
            } else if stats.genders.is_empty() {
                gender = "no-gender".to_owned()
            } else {
                gender = stats.genders.to_gender().to_string()
            };
        }
        out.write_object(ClusterGenderInfo { cluster, gender })?;
    }

    out.finish()?;

    Ok(())
}

impl Command for AuthorGender {
    fn exec(&self) -> Result<()> {
        let clusters = clusters::all_clusters("book-links/cluster-stats.parquet")?;
        let name_genders = authors::viaf_author_table()?;
        let cluster_genders = clusters::read_resolve(&self.author_file, &name_genders)?;
        save_genders(clusters, cluster_genders, self.output.as_ref())?;

        Ok(())
    }
}
