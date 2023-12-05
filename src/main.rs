use arithmetic_encoder::ArithmeticEncoder;

use crate::cli::Cli;

pub mod arithmetic_encoder;
pub mod cli;

fn main() {
  let cli = Cli::new();

  // TODO: add clippy
  // let my_array = vec![[1], [2], [3], [4], [5]].into_iter().map(|x| [x[0] * 2]).flatten();

  let operation = cli.operation();
  let filename = cli.filename;

  let lower_bound = cli.lower_bound;
  let upper_bound = cli.upper_bound;
  let output_filename = cli.output;

  let mut encoder = ArithmeticEncoder::new(lower_bound, upper_bound);

  match operation {
    cli::Operation::Encode => {
      // print the maximum value of u32
      encoder.encode(filename, output_filename).unwrap();
    }
    cli::Operation::Decode => {
      encoder.decode(filename).unwrap();
    }
  }
}
