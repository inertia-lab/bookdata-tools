use indicatif::ProgressBar;
use std::fs;
use std::io::{BufRead, BufReader, Result as IOResult};
use std::path::Path;

pub mod background;
pub mod compress;
pub mod ext;
pub mod lines;
pub mod object;

pub use compress::open_gzin_progress;
pub use lines::LineProcessor;
pub use object::ObjectWriter;

/// Open a file as a buffered reader with a progress bar.
pub fn open_progress(path: &Path, pb: ProgressBar) -> IOResult<impl BufRead> {
    let name = path.file_name().unwrap().to_string_lossy();
    let read = fs::File::open(path)?;
    pb.set_length(read.metadata()?.len());
    pb.set_prefix(name.to_string());

    let read = pb.wrap_read(read);
    let read = BufReader::new(read);
    Ok(read)
}
