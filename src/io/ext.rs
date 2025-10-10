use std::fs::File;
use std::io;
use std::io::Read;
use zip::read::ZipFile;

use log::*;

/// Trait for read types that can report their size.
pub trait LengthRead: Read {
    fn input_size(&self) -> io::Result<u64>;

    /// Read all bytes from a file, using its size to pre-allocate capacity.
    fn read_all_sized(&mut self) -> io::Result<Vec<u8>> {
        let mut cap = self.input_size()? as usize;
        let mut out = vec![0; cap + 1];
        let mut pos = 0;
        loop {
            trace!("requesting {} bytes", cap - pos);
            let ls = self.read(&mut out[pos..])?;
            pos += ls;
            if pos >= cap {
                // the size hint was wrong
                warn!("size was wrong, expanding");
                cap += 16 * 1024;
                out.resize(cap, 0);
            }
            if ls == 0 {
                out.truncate(pos);
                return Ok(out);
            }
        }
    }
}

impl LengthRead for File {
    fn input_size(&self) -> io::Result<u64> {
        let meta = self.metadata()?;
        Ok(meta.len())
    }
}

impl<'a, R: Read> LengthRead for ZipFile<'a, R> {
    fn input_size(&self) -> io::Result<u64> {
        Ok(self.size())
    }
}
