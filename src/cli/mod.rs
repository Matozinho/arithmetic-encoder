use clap::{Parser, ValueEnum};
use std::{
  fmt::{self, Display},
  path::PathBuf,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
  /// Encode file (default)
  #[arg(short, long, action, default_value_t = true)]
  pub encode: bool,

  /// Decode file
  #[arg(short, long, action, help = "Decode the input file")]
  pub decode: bool,

  /// the minimum value to encode. Only used when encoding
  #[arg(short, long, default_value_t = 0)]
  pub lower_bound: u32,

  /// the maximum value to encode. Only used when encoding
  #[arg(short, long, default_value_t = u32::MAX)]
  pub upper_bound: u32,

  /// the name of the output file
  #[arg(short, long, default_value = "output")]
  pub output: PathBuf,

  /// The name of the file that will be operated on
  pub filename: PathBuf,
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

  pub fn operation(&self) -> Operation {
    if self.decode {
      Operation::Decode
    } else {
      // Default to Encode if neither or both flags are set
      Operation::Encode
    }
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
