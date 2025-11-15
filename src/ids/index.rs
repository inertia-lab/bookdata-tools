//! Data structure for mapping string keys to numeric identifiers.
use std::borrow::Borrow;
use std::fs::File;
use std::hash::Hash;
use std::path::Path;
use std::sync::Arc;

use arrow::{
    array::{Int32Array, RecordBatch, StringArray},
    datatypes::{DataType, Field, Schema},
};
use hashbrown::hash_map::{HashMap, Keys};
use parquet::arrow::{arrow_reader::ParquetRecordBatchReaderBuilder, ArrowWriter, ProjectionMask};

use anyhow::{anyhow, Result};
use log::*;
use thiserror::Error;

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};
#[cfg(test)]
use tempfile::tempdir;

use crate::arrow::writer::parquet_writer_defaults;

/// The type of index identifiers.
pub type Id = i32;

#[derive(Error, Debug)]
pub enum IndexError {
    #[error("key not present in frozen index")]
    KeyNotPresent,
}

/// Index identifiers from a data type
pub struct IdIndex<K> {
    map: HashMap<K, Id>,
    frozen: bool,
}

impl<K> IdIndex<K>
where
    K: Eq + Hash,
{
    /// Create a new index.
    pub fn new() -> IdIndex<K> {
        IdIndex {
            map: HashMap::new(),
            frozen: false,
        }
    }

    /// Freeze the index so no new items can be added.
    #[allow(dead_code)]
    pub fn freeze(self) -> IdIndex<K> {
        IdIndex {
            map: self.map,
            frozen: true,
        }
    }

    /// Get the index length
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Get the ID for a key, adding it to the index if needed.
    pub fn intern<Q>(&mut self, key: &Q) -> Result<Id, IndexError>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ToOwned<Owned = K> + ?Sized,
    {
        let n = self.map.len() as Id;
        if self.frozen {
            self.lookup(key).ok_or(IndexError::KeyNotPresent)
        } else {
            // use Hashbrown's raw-entry API to minimize cloning
            let eb = self.map.raw_entry_mut();
            let e = eb.from_key(key);
            let (_, v) = e.or_insert_with(|| (key.to_owned(), n + 1));
            Ok(*v)
        }
    }

    /// Get the ID for a key, adding it to the index if needed and transferring ownership.
    pub fn intern_owned(&mut self, key: K) -> Result<Id, IndexError> {
        let n = self.map.len() as Id;
        if self.frozen {
            self.lookup(&key).ok_or(IndexError::KeyNotPresent)
        } else {
            Ok(*self.map.entry(key).or_insert(n + 1))
        }
    }

    /// Look up the ID for a key if it is present.
    #[allow(dead_code)]
    pub fn lookup<Q>(&self, key: &Q) -> Option<Id>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.get(key).map(|i| *i)
    }

    /// Iterate over keys (see [std::collections::HashMap::keys]).
    #[allow(dead_code)]
    pub fn keys(&self) -> Keys<'_, K, Id> {
        self.map.keys()
    }
}

impl IdIndex<String> {
    /// Get the keys in order.
    pub fn key_vec(&self) -> Vec<&str> {
        let mut vec = Vec::with_capacity(self.len());
        vec.resize(self.len(), None);
        for (k, n) in self.map.iter() {
            let i = (n - 1) as usize;
            assert!(vec[i].is_none());
            vec[i] = Some(k);
        }

        let vec = vec.iter().map(|ro| ro.unwrap().as_str()).collect();
        vec
    }

    /// Convert this ID index into an Arrow RecordBatch, with columns for ID and key.
    pub fn record_batch(&self, id_col: &str, key_col: &str) -> Result<RecordBatch> {
        debug!("preparing data frame for index");
        let n = self.map.len() as i32;
        let ids: Vec<_> = (0..n).collect();
        let ids = Int32Array::from(ids);
        let keys = StringArray::from(self.key_vec());

        let schema = Schema::new(vec![
            Field::new(id_col, DataType::Int32, false),
            Field::new(key_col, DataType::Utf8, false),
        ]);

        let rb = RecordBatch::try_new(Arc::new(schema), vec![Arc::new(ids), Arc::new(keys)])?;
        Ok(rb)
    }

    /// Load from a Parquet file, with a standard configuration.
    ///
    /// This assumes the Parquet file has the following columns:
    ///
    /// - `key`, of type `String`, storing the keys
    /// - `id`, of type `i32`, storing the IDs
    pub fn load_standard<P: AsRef<Path>>(path: P) -> Result<IdIndex<String>> {
        IdIndex::load(path, "id", "key")
    }

    /// Load from a Parquet file.
    ///
    /// This loads two columns from a Parquet file.  The ID column is expected to
    /// have type `UInt32` (or a type projectable to it), and the key column should
    /// be `Utf8`.
    pub fn load<P: AsRef<Path>>(path: P, id_col: &str, key_col: &str) -> Result<IdIndex<String>> {
        let path_str = path.as_ref().to_string_lossy();
        info!("reading index from file {}", path_str);
        let file = File::open(path.as_ref())?;
        let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
        let project = ProjectionMask::columns(builder.parquet_schema(), [id_col, key_col]);
        let reader = builder.with_projection(project).build()?;

        let mut map = HashMap::new();

        debug!("reading file contents");
        for batch in reader {
            let batch = batch?;
            assert_eq!(batch.schema().field(0).name(), id_col);
            assert_eq!(batch.schema().field(1).name(), key_col);
            let ic = batch
                .column(0)
                .as_any()
                .downcast_ref::<Int32Array>()
                .ok_or_else(|| {
                    anyhow!(
                        "invalid id column type {}",
                        batch.schema().field(0).data_type()
                    )
                })?;
            let kc = batch
                .column(1)
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| {
                    anyhow!(
                        "invalid id column type {}",
                        batch.schema().field(0).data_type()
                    )
                })?;
            for pair in ic.into_iter().zip(kc.into_iter()) {
                if let (Some(id), Some(key)) = pair {
                    map.insert(key.to_string(), id);
                }
            }
        }

        info!("read {} keys from {}", map.len(), path_str);

        Ok(IdIndex { map, frozen: false })
    }

    /// Save to a Parquet file with the standard configuration.
    pub fn save_standard<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.save(path, "id", "key")
    }

    /// Save to a Parquet file with the standard configuration.
    pub fn save<P: AsRef<Path>>(&self, path: P, id_col: &str, key_col: &str) -> Result<()> {
        let frame = self.record_batch(id_col, key_col)?;

        let path = path.as_ref();
        info!("saving index to {:?}", path);
        let schema = Schema::new(vec![
            Field::new(id_col, DataType::Int32, false),
            Field::new(key_col, DataType::Utf8, false),
        ]);
        let schema = Arc::new(schema);
        let file = File::create(path)?;
        let props = parquet_writer_defaults().build();
        let mut writer = ArrowWriter::try_new(file, schema, Some(props))?;
        writer.write(&frame)?;
        writer.finish()?;

        Ok(())
    }
}

#[test]
fn test_index_empty() {
    let index: IdIndex<String> = IdIndex::new();
    assert_eq!(index.len(), 0);
    assert!(index.lookup("bob").is_none());
}

#[test]
fn test_index_intern_one() {
    let mut index: IdIndex<String> = IdIndex::new();
    assert!(index.lookup("hackem muche").is_none());
    let id = index.intern("hackem muche").expect("intern failure");
    assert_eq!(id, 1);
    assert_eq!(index.lookup("hackem muche").unwrap(), 1);
}

#[test]
fn test_index_intern_two() {
    let mut index: IdIndex<String> = IdIndex::new();
    assert!(index.lookup("hackem muche").is_none());
    let id = index.intern("hackem muche");
    assert_eq!(id.expect("intern failure"), 1);
    let id2 = index.intern("readme");
    assert_eq!(id2.expect("intern failure"), 2);
    assert_eq!(index.lookup("hackem muche").unwrap(), 1);
}

#[test]
fn test_index_intern_twice() {
    let mut index: IdIndex<String> = IdIndex::new();
    assert!(index.lookup("hackem muche").is_none());
    let id = index.intern("hackem muche");
    assert_eq!(id.expect("intern failure"), 1);
    let id2 = index.intern("hackem muche");
    assert_eq!(id2.expect("intern failure"), 1);
    assert_eq!(index.len(), 1);
}

#[test]
fn test_index_intern_twice_owned() {
    let mut index: IdIndex<String> = IdIndex::new();
    assert!(index.lookup("hackem muche").is_none());
    let id = index.intern_owned("hackem muche".to_owned());
    assert!(id.is_ok());
    assert_eq!(id.expect("intern failure"), 1);
    let id2 = index.intern_owned("hackem muche".to_owned());
    assert!(id2.is_ok());
    assert_eq!(id2.expect("intern failure"), 1);
    assert_eq!(index.len(), 1);
}

#[cfg(test)]
#[test_log::test]
fn test_index_save() -> Result<()> {
    let mut index: IdIndex<String> = IdIndex::new();
    let mut gen = Gen::new(100);
    for _i in 0..10000 {
        let key = String::arbitrary(&mut gen);
        let prev = index.lookup(&key);
        let id = index.intern(&key).expect("intern failure");
        match prev {
            Some(i) => assert_eq!(id, i),
            None => assert_eq!(id as usize, index.len()),
        };
    }

    let dir = tempdir()?;
    let pq = dir.path().join("index.parquet");
    index.save_standard(&pq).expect("save error");

    let i2 = IdIndex::load_standard(&pq).expect("load error");
    assert_eq!(i2.len(), index.len());
    for (k, v) in &index.map {
        let v2 = i2.lookup(k);
        assert!(v2.is_some());
        assert_eq!(v2.unwrap(), *v);
    }

    Ok(())
}

#[test]
fn test_index_freeze() {
    let mut index: IdIndex<String> = IdIndex::new();
    assert!(index.lookup("hackem muche").is_none());
    let id = index.intern("hackem muche");
    assert!(id.is_ok());
    assert_eq!(id.expect("intern failure"), 1);

    let mut index = index.freeze();

    let id = index.intern("hackem muche");
    assert!(id.is_ok());
    assert_eq!(id.expect("intern failure"), 1);

    let id2 = index.intern("foobie bletch");
    assert!(id2.is_err());
}
