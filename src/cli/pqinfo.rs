//! Extract basic information from a Parquet file.
use std::fmt::Debug;
use std::fs::File;
use std::io::{stdout, Write};
use std::mem::drop;

use arrow::datatypes::Field;
use friendly::{bytes, scalar};
use parquet::arrow::parquet_to_arrow_schema;
use parquet::file::reader::{FileReader, SerializedFileReader};
use serde::Serialize;

use crate::prelude::*;

use super::Command;

/// Extract basic information from a Parquet file.
#[derive(Args, Debug)]
#[command(name = "collect-isbns")]
pub struct PQInfo {
    /// Path to the output JSON file.
    #[arg(short = 'o', long = "output")]
    out_file: Option<PathBuf>,

    /// Path to the Parquet file.
    #[arg(name = "FILE")]
    source_file: PathBuf,
}

#[derive(Debug, Serialize)]
struct InfoStruct {
    row_count: i64,
    file_size: u64,
    fields: Vec<FieldStruct>,
}

#[derive(Debug, Serialize)]
struct FieldStruct {
    name: String,
    data_type: String,
    nullable: bool,
}

impl From<&Field> for FieldStruct {
    fn from(f: &Field) -> Self {
        FieldStruct {
            name: f.name().clone(),
            data_type: format!("{:?}", f.data_type()),
            nullable: f.is_nullable(),
        }
    }
}

impl Command for PQInfo {
    fn exec(&self) -> Result<()> {
        info!("reading {:?}", self.source_file);

        let pqf = File::open(&self.source_file)?;
        let fmeta = pqf.metadata()?;
        info!("file size: {}", bytes(fmeta.len()));

        let file = SerializedFileReader::new(pqf)?;
        let meta = file.metadata();

        let n_rows = meta.file_metadata().num_rows();
        info!("row count: {}", scalar(n_rows));
        info!("row groups: {}", meta.row_groups().len());
        let rc2: i64 = meta.row_groups().iter().map(|rg| rg.num_rows()).sum();
        if rc2 != n_rows {
            warn!("row group total {} != file total {}", rc2, n_rows);
        }

        info!("decoding schema");
        let schema = meta.file_metadata().schema_descr();
        let schema = parquet_to_arrow_schema(schema, meta.file_metadata().key_value_metadata())?;

        let out = stdout();
        let mut ol = out.lock();
        for field in &schema.fields {
            writeln!(&mut ol, "{:?}", field)?;
        }
        drop(ol);

        if let Some(ref file) = self.out_file {
            let mut out = File::create(file)?;
            let info = InfoStruct {
                row_count: n_rows,
                file_size: fmeta.len() as u64,
                fields: schema.fields.iter().map(|f| f.as_ref().into()).collect(),
            };

            serde_json::to_writer_pretty(&mut out, &info)?;
            out.write(b"\n")?;
        }

        Ok(())
    }
}
