use crate::goodreads::*;
use crate::io::object::{ChunkWriter, ThreadObjectWriter, UnchunkWriter};
use crate::prelude::*;
use crate::util::logging::data_progress;
use serde::de::DeserializeOwned;

#[derive(clap::Subcommand, Debug)]
pub enum GRScan {
    /// Scan GoodReads works.
    Works(ScanInput),
    /// Scan GoodReads books.
    Books(ScanInput),
    /// Scan GoodReads genres.
    Genres(ScanInput),
    /// Scan GoodReads authors.
    Authors(ScanInput),
    /// Scan GoodReads interactions.
    Interactions(ScanInput),
    /// Scan GoodReads reviews.
    Reviews(ScanInput),
}

#[derive(Args, Debug)]
pub struct ScanInput {
    /// Input file
    #[arg(name = "INPUT")]
    infile: PathBuf,
}

fn scan_gr<R, W>(path: &Path, proc: W) -> Result<()>
where
    W: ObjectWriter<R> + Send + Sync + 'static,
    R: DeserializeOwned + Send + Sync + 'static,
{
    info!("reading data from {}", path.display());
    let pb = data_progress(0);
    let read = LineProcessor::open_gzip(path, pb.clone())?;
    let proc = ChunkWriter::new(proc);
    let writer = ThreadObjectWriter::wrap(proc).with_name("output").spawn();
    let mut writer = UnchunkWriter::new(writer);
    read.process_json(&mut writer)?;
    pb.finish_and_clear();

    writer.finish_objects()?;

    Ok(())
}

impl GRScan {
    pub fn exec(&self) -> Result<()> {
        match self {
            GRScan::Works(opts) => {
                info!("scanning GoodReads works");
                scan_gr(&opts.infile, work::WorkWriter::open()?)?;
            }
            GRScan::Books(opts) => {
                info!("scanning GoodReads books");
                scan_gr(&opts.infile, book::BookWriter::open()?)?;
            }
            GRScan::Genres(opts) => {
                info!("scanning GoodReads book genres");
                scan_gr(&opts.infile, genres::BookGenreWriter::open()?)?;
            }
            GRScan::Authors(opts) => {
                info!("scanning GoodReads book genres");
                scan_gr(&opts.infile, author::AuthorWriter::open()?)?;
            }
            GRScan::Interactions(opts) => {
                info!("scanning GoodReads interactions");
                scan_gr(&opts.infile, interaction::IntWriter::open()?)?;
            }
            GRScan::Reviews(opts) => {
                info!("scanning GoodReads reviews");
                scan_gr(&opts.infile, review::ReviewWriter::open()?)?;
            }
        };

        Ok(())
    }
}
