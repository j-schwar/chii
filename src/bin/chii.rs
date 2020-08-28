use anyhow::{anyhow, Result};
use bit_vec::BitVec;
use chii::schema::Schema;
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
  name = "chii",
  about = "A compression utility for domain specific data"
)]
struct Opt {
  /// Uncompress file
  #[structopt(short, long)]
  decompress: bool,

  /// Print compressed object blocks
  #[structopt(long)]
  blocks: bool,

  /// Output file
  #[structopt(short)]
  out_file: Option<PathBuf>,

  /// Path to the data schema
  schema: PathBuf,

  /// Path to the data
  file: PathBuf,
}

impl Opt {
  fn output_file_path(&self) -> PathBuf {
    if let Some(path) = &self.out_file {
      path.clone()
    } else {
      let mut input_file = self.file.clone();
      input_file.set_extension("co");
      input_file
    }
  }
}

fn compress(opt: &Opt) -> Result<()> {
  // Load schema from file
  let schema_file = File::open(&opt.schema)?;
  let schema: Schema = serde_yaml::from_reader(schema_file)?;

  // Load data from file
  let data_file = File::open(&opt.file)?;
  let data: Value = serde_json::from_reader(data_file)?;

  // Perform compression
  let co = chii::encode(&schema, &data)?;
  if opt.blocks {
    for block in &co.blocks {
      println!("{}", block);
    }
  }

  let bits: BitVec = co.into();
  let bytes = bits.to_bytes();

  // Write to output file
  let mut file = File::create(opt.output_file_path())?;
  file.write_all(&bytes)?;

  Ok(())
}

fn main() -> Result<()> {
  let opt = Opt::from_args();
  if opt.decompress {
    Err(anyhow!("decompression is not supported yet"))
  } else {
    compress(&opt)
  }
}
