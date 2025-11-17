use std::path::Path;

use anyhow::Result;
use parquet_derive::{ParquetRecordReader, ParquetRecordWriter};

use super::record::*;
use crate::arrow::*;
use crate::io::*;

/// Flat MARC field record.
#[derive(ParquetRecordWriter, ParquetRecordReader, Debug, Default)]
pub struct FieldRecord {
    pub rec_id: u32,
    pub fld_no: u32,
    pub tag: i16,
    pub ind1: u8,
    pub ind2: u8,
    pub sf_code: u8,
    pub contents: String,
}

/// Output for writing flat MARC fields to Parquet.
pub struct FieldOutput {
    rec_count: u32,
    writer: TableWriter<FieldRecord>,
}

impl FieldOutput {
    /// Create a new output.
    pub fn new(writer: TableWriter<FieldRecord>) -> FieldOutput {
        FieldOutput {
            rec_count: 0,
            writer,
        }
    }

    /// Open a field output going to a file.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<FieldOutput> {
        let writer = TableWriter::open(path)?;
        Ok(Self::new(writer))
    }
}

impl ObjectWriter<MARCRecord> for FieldOutput {
    fn write_object(&mut self, rec: MARCRecord) -> Result<()> {
        self.rec_count += 1;
        let rec_id = self.rec_count;
        let mut fld_no = 0;

        // write the leader
        self.writer.write_object(FieldRecord {
            rec_id,
            fld_no,
            tag: -1,
            ind1: 0.into(),
            ind2: 0.into(),
            sf_code: 0.into(),
            contents: rec.leader,
        })?;

        // write the control fields
        for cf in rec.control {
            fld_no += 1;
            self.writer.write_object(FieldRecord {
                rec_id,
                fld_no,
                tag: cf.tag.into(),
                ind1: 0.into(),
                ind2: 0.into(),
                sf_code: 0.into(),
                contents: cf.content,
            })?;
        }

        // write the data fields
        for df in rec.fields {
            for sf in df.subfields {
                fld_no += 1;
                self.writer.write_object(FieldRecord {
                    rec_id,
                    fld_no,
                    tag: df.tag,
                    ind1: df.ind1.into(),
                    ind2: df.ind2.into(),
                    sf_code: sf.code.into(),
                    contents: sf.content,
                })?;
            }
        }

        Ok(())
    }

    fn finish_objects(self) -> Result<usize> {
        self.writer.finish_objects()
    }
}
