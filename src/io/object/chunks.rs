use std::{marker::PhantomData, mem::replace};

use anyhow::Result;

use super::ObjectWriter;

/// Wrap an object writer so it can receive chunks.
pub struct ChunkWriter<T, W>
where
    W: ObjectWriter<T>,
{
    _phantom: PhantomData<T>,
    delegate: W,
}

impl<T, W> ChunkWriter<T, W>
where
    W: ObjectWriter<T>,
{
    pub fn new(delegate: W) -> ChunkWriter<T, W> {
        ChunkWriter {
            _phantom: PhantomData,
            delegate,
        }
    }
}

impl<T, W> ObjectWriter<Vec<T>> for ChunkWriter<T, W>
where
    W: ObjectWriter<T>,
{
    fn write_object(&mut self, chunk: Vec<T>) -> Result<()> {
        self.delegate.write_all_objects(chunk.into_iter())?;
        Ok(())
    }

    fn finish_objects(self) -> Result<usize> {
        self.delegate.finish_objects()
    }
}

/// Wrap a writer of chunks to take individual values.
pub struct UnchunkWriter<T, W>
where
    W: ObjectWriter<Vec<T>>,
{
    delegate: W,
    chunk: Vec<T>,
    chunk_size: usize,
}

impl<T, W> UnchunkWriter<T, W>
where
    W: ObjectWriter<Vec<T>>,
{
    #[allow(dead_code)]
    pub fn new(delegate: W) -> UnchunkWriter<T, W> {
        UnchunkWriter::with_size(delegate, 1024)
    }

    pub fn with_size(delegate: W, size: usize) -> UnchunkWriter<T, W> {
        UnchunkWriter {
            delegate,
            chunk: Vec::with_capacity(size),
            chunk_size: size,
        }
    }
}

impl<T, W> ObjectWriter<T> for UnchunkWriter<T, W>
where
    W: ObjectWriter<Vec<T>>,
{
    fn write_object(&mut self, obj: T) -> Result<()> {
        self.chunk.push(obj);
        if self.chunk.len() >= self.chunk_size {
            let chunk = replace(&mut self.chunk, Vec::with_capacity(self.chunk_size));
            self.delegate.write_object(chunk)?;
        }
        Ok(())
    }

    fn finish_objects(mut self) -> Result<usize> {
        if self.chunk.len() > 0 {
            self.delegate.write_object(self.chunk)?;
        }
        self.delegate.finish_objects()
    }
}
