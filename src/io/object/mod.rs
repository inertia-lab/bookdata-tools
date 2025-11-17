use std::io::prelude::*;
use std::mem::drop;

use anyhow::Result;
use csv;
use serde::Serialize;

mod chunks;
mod thread;

pub use chunks::{ChunkWriter, UnchunkWriter};
pub use thread::ThreadObjectWriter;

/// Trait for writing objects to some kind of sink.
pub trait ObjectWriter<T>: Sized {
    /// Write one object.
    fn write_object(&mut self, object: T) -> Result<()>;

    /// Write an iterator full of objects.
    fn write_all_objects<I>(&mut self, objects: I) -> Result<usize>
    where
        I: Iterator<Item = T>,
    {
        let mut count = 0;
        for obj in objects {
            self.write_object(obj)?;
            count += 1;
        }
        Ok(count)
    }

    /// Finish and close the target.
    fn finish_objects(self) -> Result<usize>;
}

/// References can be used as object writers; however, [ObjectWriter::finish] must
/// be called on the owned writer, not a reference.  Closing the reference is a
/// no-op.
impl<T, W> ObjectWriter<T> for &mut W
where
    W: ObjectWriter<T>,
{
    fn write_object(&mut self, object: T) -> Result<()> {
        (**self).write_object(object)
    }

    fn finish_objects(self) -> Result<usize> {
        // closing a reference does *not* close the write
        Ok(0)
    }
}

impl<T: Serialize, W: Write> ObjectWriter<T> for csv::Writer<W> {
    fn write_object(&mut self, object: T) -> Result<()> {
        self.serialize(object)?;
        Ok(())
    }

    fn finish_objects(mut self) -> Result<usize> {
        self.flush()?;
        drop(self);
        Ok(0)
    }
}
