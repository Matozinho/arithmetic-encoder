use clap::{Parser, ValueEnum};
use std::{
  fmt::{self, Display},
  path::PathBuf,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
  /// the name of the file that will be operated on
  #[arg(short, long, name = "FILE")]
  pub filename: PathBuf,

  /// the operation to perform
  #[arg(short, long, default_value_t = Operation::Encode)]
  pub operation: Operation,

  /// the minimum value to encode
  #[arg(long, default_value_t = 0)]
  pub encode_min: u32,

  /// the maximum value to encode
  #[arg(long, default_value_t = u32::MAX)]
  pub encode_max: u32,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Operation {
  Encode,
  Decode,
}

impl Cli {
  pub fn new() -> Self {
    Self::parse()
  }
}

// Implementing `Display` for `Operation`
impl Display for Operation {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Operation::Encode => write!(f, "encode"),
      Operation::Decode => write!(f, "decode"),
    }
  }
}
