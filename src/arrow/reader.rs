//! Support for streaming objects from a Parquet file.
use std::fs::File;
use std::path::Path;
use std::thread::spawn;

use anyhow::Result;
use crossbeam::channel::{bounded, Receiver, Sender};
use indicatif::ProgressBar;
use log::*;
use parquet::file::reader::{ChunkReader, FileReader};
use parquet::file::serialized_reader::SerializedFileReader;
use parquet::record::RecordReader;

use crate::util::logging::{item_progress, measure_and_send, meter_bar};

/// Iterator over deserialized records from a Parquet file.
pub struct RecordIter<R>
where
    R: Send + Sync + 'static,
    Vec<R>: RecordReader<R>,
{
    remaining: usize,
    channel: Receiver<Result<Vec<R>>>,
    batch: Option<std::vec::IntoIter<R>>,
}

impl<R> RecordIter<R>
where
    R: Send + Sync + 'static,
    Vec<R>: RecordReader<R>,
{
    pub fn remaining(&self) -> usize {
        self.remaining
    }
}

/// Scan a Parquet file in a background thread and deserialize records.
pub fn scan_parquet_file<R, P>(path: P) -> Result<RecordIter<R>>
where
    P: AsRef<Path>,
    R: Send + Sync + 'static,
    Vec<R>: RecordReader<R>,
{
    let path = path.as_ref();
    let reader = File::open(&path)?;
    let reader = SerializedFileReader::new(reader)?;
    let meta = reader.metadata().file_metadata();
    let row_count = meta.num_rows();

    info!(
        "scanning {} with {} rows",
        path.display(),
        friendly::scalar(row_count)
    );
    let pb = item_progress(row_count, &format!("{}", path.display()));

    // use a small bound since we're sending whole batches
    let (send, receive) = bounded(5);
    let p2 = path.to_path_buf();

    spawn(move || {
        let send = send;
        if let Err(e) = scan_backend(reader, &send, &p2, pb) {
            send.send(Err(e)).expect("failed to send error message");
        }
    });

    debug!(
        "{}: background thread spawned, returning iter",
        path.display()
    );
    Ok(RecordIter {
        remaining: row_count as usize,
        channel: receive,
        batch: None,
    })
}

impl<R> Iterator for RecordIter<R>
where
    R: Send + Sync + 'static,
    Vec<R>: RecordReader<R>,
{
    type Item = Result<R>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // try to get another item from the current batch
            let next = self.batch.as_mut().map(|i| i.next()).flatten();
            if let Some(row) = next {
                // we got something! return it
                self.remaining -= 1;
                return Some(Ok(row));
            } else if let Ok(br) = self.channel.recv() {
                // fetch a new batch and try again
                match br {
                    Ok(batch) => {
                        self.batch = Some(batch.into_iter());
                        // loop around and use the batch
                    }
                    Err(e) => {
                        // error on the channel
                        return Some(Err(e));
                    }
                }
            } else {
                // we're done
                return None;
            }
        }
    }
}

fn scan_backend<R, E>(
    reader: SerializedFileReader<R>,
    send: &Sender<Result<Vec<E>>>,
    path: &Path,
    pb: ProgressBar,
) -> Result<()>
where
    R: ChunkReader + 'static,
    E: Send + Sync + 'static,
    Vec<E>: RecordReader<E>,
{
    debug!("{}: beginning batched read", path.display());
    let meter = meter_bar(5, &format!("{} read buffer", path.display()));
    let ngroups = reader.num_row_groups();
    for gi in 0..ngroups {
        let mut group = reader.get_row_group(gi)?;
        let gmeta = group.metadata();
        let glen = gmeta.num_rows();

        trace!("{}: received batch of {} rows", path.display(), glen);
        pb.inc(glen as u64);
        let mut batch = Vec::with_capacity(glen as usize);
        batch.read_from_row_group(group.as_mut(), glen as usize)?;
        assert_eq!(batch.len(), glen as usize);
        trace!("{}: decoded chunk, sending to consumer", path.display());
        measure_and_send(&send, Ok(batch), &meter)?;
    }
    Ok(())
}
