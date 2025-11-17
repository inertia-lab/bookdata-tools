use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::io::LineProcessor;
use crate::openlib::*;
use crate::prelude::*;
use crate::util::logging::data_progress;

use super::Command;

#[derive(Args, Debug)]
struct Input {
    /// Input file
    #[arg(name = "INPUT")]
    infile: PathBuf,
}

#[derive(clap::Subcommand, Debug)]
enum DataType {
    /// Parse OpenLibrary works.
    ///
    /// Authors must be processed first.
    ScanWorks(Input),

    /// Parse OpenLibrary editions.
    ///
    /// Authors and works must be processed first.
    ScanEditions(Input),

    /// Parse OpenLibrary authors.
    ScanAuthors(Input),
}

/// Scan OpenLibrary data.
#[derive(Args, Debug)]
#[command(name = "openlib")]
pub struct OpenLib {
    #[command(subcommand)]
    mode: DataType,
}

/// Helper function to route OpenLibrary data.
fn scan_openlib<R, Proc>(path: &Path, proc: Proc) -> Result<()>
where
    Proc: ObjectWriter<Row<R>>,
    R: DeserializeOwned,
{
    let mut proc = proc;
    let mut nlines = 0;
    info!("opening file {}", path.to_string_lossy());
    let pb = data_progress(0);
    let input = LineProcessor::open_gzip(path, pb.clone())?;

    for line in input.records() {
        nlines += 1;
        if !line.is_ok() {
            error!("parse error on line {}", nlines);
        }
        let row: Row<R> = line?;
        proc.write_object(row)?;
    }

    proc.finish_objects()?;

    Ok(())
}

impl Command for OpenLib {
    fn exec(&self) -> Result<()> {
        match &self.mode {
            DataType::ScanAuthors(opts) => {
                scan_openlib(&opts.infile, AuthorProcessor::new()?)?;
            }
            DataType::ScanWorks(opts) => {
                scan_openlib(&opts.infile, WorkProcessor::new()?)?;
            }
            DataType::ScanEditions(opts) => {
                scan_openlib(&opts.infile, EditionProcessor::new()?)?;
            }
        };

        Ok(())
    }
}
