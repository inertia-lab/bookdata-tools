use std::{
    borrow::Cow,
    thread::{spawn, JoinHandle, Scope, ScopedJoinHandle},
};

use anyhow::Result;
use crossbeam::channel::{bounded, Receiver, Sender};
use indicatif::ProgressBar;

use crate::util::logging::{measure_and_recv, measure_and_send, meter_bar};

use super::ObjectWriter;

enum WorkHandle<'scope> {
    Static(JoinHandle<Result<usize>>),
    Scoped(ScopedJoinHandle<'scope, Result<usize>>),
}

fn ferry<T, W>(recv: Receiver<T>, writer: W, pb: ProgressBar) -> Result<usize>
where
    T: Send + Sync + 'static,
    W: ObjectWriter<T>,
{
    let mut writer = writer; // move writer into thread

    while let Some(obj) = measure_and_recv(&recv, &pb) {
        writer.write_object(obj)?;
    }

    pb.finish_and_clear();
    writer.finish_objects()
}

pub struct ThreadObjectWriterBuilder<W, F: Send + FnOnce() -> Result<W>> {
    thunk: F,
    name: String,
    capacity: usize,
}

/// Write objects in a background thread.
pub struct ThreadObjectWriter<'scope, T>
where
    T: Send + Sync + 'static,
{
    sender: Sender<T>,
    handle: WorkHandle<'scope>,
    meter: ProgressBar,
}

impl<'scope, T> ThreadObjectWriter<'scope, T>
where
    T: Send + Sync + 'scope,
{
    pub fn wrap<W>(writer: W) -> ThreadObjectWriterBuilder<W, impl Send + FnOnce() -> Result<W>>
    where
        W: ObjectWriter<T> + Send + Sync + 'scope,
    {
        ThreadObjectWriterBuilder {
            thunk: move || Ok(writer),
            name: "unnamed".into(),
            capacity: 100,
        }
    }
}

impl<W, F: Send + FnOnce() -> Result<W>> ThreadObjectWriterBuilder<W, F> {
    /// Set the channel capacity for this thread writer.  Defaults to 100.
    pub fn with_capacity(self, cap: usize) -> Self {
        ThreadObjectWriterBuilder {
            capacity: cap,
            ..self
        }
    }

    /// Set a name for this thread writer for debugging.
    pub fn with_name<S: Into<Cow<'static, str>>>(self, name: S) -> Self {
        let name: Cow<'static, str> = name.into();
        ThreadObjectWriterBuilder {
            name: name.to_string(),
            ..self
        }
    }

    /// Spawn the thread writer.
    pub fn spawn_scoped<'scope, 'env, T>(
        self,
        scope: &'scope Scope<'scope, 'env>,
    ) -> ThreadObjectWriter<'scope, T>
    where
        W: ObjectWriter<T> + 'scope,
        F: 'scope,
        T: Send + Sync + 'scope,
    {
        let (sender, receiver) = bounded(self.capacity);
        let pb = meter_bar(self.capacity, &format!("{} buffer", self.name));

        let rpb = pb.clone();
        let thunk = self.thunk;
        let h = scope.spawn(move || ferry(receiver, thunk()?, rpb));

        ThreadObjectWriter {
            meter: pb,
            sender,
            handle: WorkHandle::Scoped(h),
        }
    }

    /// Spawn the thread writer.
    pub fn spawn<T>(self) -> ThreadObjectWriter<'static, T>
    where
        W: ObjectWriter<T> + 'static,
        F: 'static,
        T: Send + Sync + 'static,
    {
        let (sender, receiver) = bounded(self.capacity);
        let pb = meter_bar(self.capacity, &format!("{} buffer", self.name));

        let rpb = pb.clone();
        let thunk = self.thunk;
        let h = spawn(move || ferry(receiver, thunk()?, rpb));

        ThreadObjectWriter {
            meter: pb,
            sender,
            handle: WorkHandle::Static(h),
        }
    }
}

impl<'scope, T: Send + Sync + 'scope> ThreadObjectWriter<'scope, T> {
    /// Create a satellite writer that writes to the same backend as this
    /// writer.
    ///
    /// Satellites can be used to enable multiple data-generating threads to
    /// write to the same thread writer, turning it into a multi-producer,
    /// single-consumer writing pipeline.  Satellite writers should be finished,
    /// and closing them does not finish the original thread writer (it still
    /// needs to have [ObjectWriter::finish] called, typically after all
    /// satellites are done, but it calling [ObjectWriter::finish] while
    /// satellites are still active will wait until the satellites have finished
    /// and closed their connections to the consumer thread).
    ///
    /// Satellites hold a reference to the original thread writer, to discourage
    /// keeping them alive after the thread writer has been finished.  They
    /// work best with [std::thread::scope]:
    ///
    /// ```rust,ignore
    /// let writer = ThreadWriter::new(writer);
    /// scope(|s| {
    ///     for i in 0..NTHREADS {
    ///         let out = writer.satellite();
    ///         s.spawn(move || {
    ///             // process and write to out
    ///             out.finish().expect("closing writer failed");
    ///         })
    ///     }
    /// })
    /// ```
    pub fn satellite<'a>(&'a self) -> ThreadWriterSatellite<'a, 'scope, T>
    where
        'scope: 'a,
    {
        ThreadWriterSatellite::create(self)
    }
}

impl<'scope, T: Send + Sync + 'static> ObjectWriter<T> for ThreadObjectWriter<'scope, T> {
    fn write_object(&mut self, object: T) -> Result<()> {
        measure_and_send(&self.sender, object, &self.meter)?;
        Ok(())
    }

    fn finish_objects(self) -> Result<usize> {
        // dropping the sender will cause the consumer to hang up.
        drop(self.sender);
        // wait for the consumer to finish before we consider the writer closed.
        let res = match self.handle {
            WorkHandle::Static(h) => h.join().map_err(std::panic::resume_unwind)?,
            WorkHandle::Scoped(h) => h.join().map_err(std::panic::resume_unwind)?,
        };
        res
    }
}

/// Child writer to allow objects to be written to a threaded writer from multiple
/// threads, allowing parallel feeding of data.  It takes a **reference** to the
/// original thread writer, so it needs to be used with [std::thread::scope]. The
/// original writer still needs to be closed, and this design ensures that the
/// original cannot be closed until all the children are finished.
#[derive(Clone)]
pub struct ThreadWriterSatellite<'a, 'scope, T>
where
    T: Send + Sync + 'static,
    'scope: 'a,
{
    delegate: &'a ThreadObjectWriter<'scope, T>,
    sender: Sender<T>,
}

impl<'a, 'scope, T> ThreadWriterSatellite<'a, 'scope, T>
where
    T: Send + Sync + 'static,
    'scope: 'a,
{
    /// Create a new thread child writer.
    fn create(delegate: &'a ThreadObjectWriter<'scope, T>) -> ThreadWriterSatellite<'a, 'scope, T> {
        ThreadWriterSatellite {
            delegate,
            sender: delegate.sender.clone(),
        }
    }
}

impl<'a, 'scope, T> ObjectWriter<T> for ThreadWriterSatellite<'a, 'scope, T>
where
    T: Send + Sync + 'static,
    'scope: 'a,
{
    fn write_object(&mut self, object: T) -> Result<()> {
        measure_and_send(&self.sender, object, &self.delegate.meter)?;
        Ok(())
    }

    fn finish_objects(self) -> Result<usize> {
        // do nothing, dropping the writer will close the sender
        Ok(0)
    }
}
