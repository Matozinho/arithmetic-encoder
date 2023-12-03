use arithmetic_encoder::ArithmeticEncoder;

use crate::cli::Cli;

pub mod arithmetic_encoder;
pub mod cli;

fn main() {
  let cli = Cli::new();

  // TODO: add clippy
  // let my_array = vec![[1], [2], [3], [4], [5]].into_iter().map(|x| [x[0] * 2]).flatten();

  let filename = cli.filename;
  let operation: cli::Operation = cli.operation;

  let encode_max = cli.encode_max;
  let encode_min = cli.encode_min;

  let encoder = ArithmeticEncoder::new(encode_min, encode_max);

  match operation {
    cli::Operation::Encode => {
      // print the maximum value of u32
      encoder.encode(filename)
    }
    cli::Operation::Decode => encoder.decode(filename),
  }
}
