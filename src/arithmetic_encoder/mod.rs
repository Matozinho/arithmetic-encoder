use std::{
  cmp::Ordering::Equal,
  collections::HashMap,
  fs::File,
  io::{BufReader, Read, Result, Seek, SeekFrom, Write},
  path::PathBuf,
};

const BUFFERING_SIZE: u16 = 10;

pub struct ArithmeticEncoder {
  lower_bound: u32,
  upper_bound: u32,
  _precision: f64,
}

impl ArithmeticEncoder {
  pub fn new(lower_bound: u32, upper_bound: u32) -> ArithmeticEncoder {
    ArithmeticEncoder {
      lower_bound,
      upper_bound,
      _precision: 10f64.powi((upper_bound.ilog10() as i32).max(0)) * 10.0,
    }
  }

  fn first_digit(number: u32) -> u8 {
    number
      .to_string()
      .chars()
      .next()
      .unwrap()
      .to_digit(10)
      .unwrap() as u8
  }

  fn calculate_probabilities(&self, reader: &mut BufReader<File>) -> Result<Vec<(u8, f64)>> {
    reader.seek(SeekFrom::Start(0))?;
    let mut frequency_map: HashMap<u8, u64> = HashMap::new();
    let mut total_bytes = 0u64;

    // Read the file byte by byte
    for byte in reader.bytes() {
      let byte = byte?;
      *frequency_map.entry(byte).or_insert(0) += 1;
      total_bytes += 1;
    }

    // Convert frequencies to probabilities and store them in a vector
    let mut probabilities: Vec<(u8, f64)> = frequency_map
      .into_iter()
      .map(|(byte, count)| (byte, count as f64 / total_bytes as f64))
      .collect();

    // Sort the vector in descending order by probability. if probabilities are equal, sort by byte value
    probabilities.sort_by(|a, b| {
      b.1
        .partial_cmp(&a.1)
        .unwrap_or(Equal)
        .then_with(|| a.0.cmp(&b.0))
    });

    Ok(probabilities)
  }

  fn remove_trailing_zeros(&self, number: u32) -> u32 {
    let binding = number.to_string();
    let trimmed_str = binding.trim_end_matches('0');
    if trimmed_str.is_empty() {
      // If all digits were zeros, return 0
      0
    } else {
      trimmed_str.parse::<u32>().unwrap()
    }
  }

  fn get_minimal_integer(&self, mut number: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    while number != 0 {
      bytes.push((number & 0xFF) as u8); // Extract the lowest 8 bits
      number >>= 8; // Shift right by 8 bits
    }

    if bytes.is_empty() {
      bytes.push(0);
    }
    bytes
  }

  // TODO: Find a way to do a buffer to write to the file
  // TODO: store the memory of the last in the output file (???)
  // TODO: store the propabilities into the output file
  fn encode_process(
    &self,
    sorted_probabilities: &Vec<(u8, f64)>,
    reader: &mut BufReader<File>,
    output_path: PathBuf,
  ) -> Result<()> {
    let mut output_vec: Vec<u8> = Vec::new();
    let mut output = File::create(output_path.with_extension("ac"))?;
    // Compute cumulative probabilities
    let mut cumulative_probabilities: Vec<(u8, f64)> = Vec::new();
    let mut cumulative = 0.0;

    for (byte, probability) in sorted_probabilities.into_iter() {
      cumulative += probability;
      cumulative_probabilities.push((*byte, cumulative));
    }

    println!("====================");
    for (byte, probability) in cumulative_probabilities.clone() {
      println!(
        "Byte: {:?}, Probability: {:.2}%",
        byte as char,
        probability * 100.0
      );
    }
    println!("====================");

    let mut low = self.lower_bound;
    let mut high = self.upper_bound;
    // let mut range;

    reader.seek(SeekFrom::Start(0))?;
    // Read the file byte by byte
    for byte_result in reader.bytes() {
      // find the byte in the cumulative probabilities
      let byte = byte_result?;
      let (index, &(_, byte_high_prob)) = cumulative_probabilities
        .iter()
        .enumerate()
        .find(|&(_, (b, _))| b == &byte)
        .unwrap();

      let byte_low_prob = if index == 0 {
        0.0
      } else {
        cumulative_probabilities[index - 1].1
      };

      let range = (high - low) as f64 + 1.0;
      let prob_in_range_high = (range * byte_high_prob).trunc() as u32;
      let prob_in_range_low = (range * byte_low_prob).trunc() as u32 as u32;

      let mut new_low = low + prob_in_range_low;
      let mut new_high = low + prob_in_range_high - 1;

      loop {
        let most_significant_high = ArithmeticEncoder::first_digit(new_high as u32);
        let most_significant_low = ArithmeticEncoder::first_digit(new_low as u32);

        if most_significant_high != most_significant_low {
          break;
        }

        // Calculate the power of 10 to remove the most significant digit
        let exponent = new_high.ilog10() as u32;
        let power_of_10 = 10u32.pow(exponent.max(0));

        // Remove the most significant digit
        new_high = (new_high - (most_significant_high as u32) * power_of_10) * 10 + 9;
        new_low = (new_low - (most_significant_low as u32) * power_of_10) * 10;

        // write the most significant digit as a string to the output
        output_vec.push(most_significant_high);
        output.write_all(&[most_significant_high])?;
      }

      low = new_low;
      high = new_high;
    }

    let mut bytes = self.get_minimal_integer(self.remove_trailing_zeros(low));

    output.write_all(&bytes)?;
    output_vec.append(&mut bytes);

    println!("{:?}", output_vec);

    Ok(())
  }

  pub fn encode(&self, filename: PathBuf, output_filename: PathBuf) -> Result<()> {
    // Open the file as a binary file
    let file = File::open(&filename).unwrap();

    let mut reader = BufReader::new(file);

    let probabilities = self.calculate_probabilities(&mut reader).unwrap();
    for (byte, probability) in probabilities.clone() {
      println!(
        "Byte: {:?}, Probability: {:.2}%",
        byte as char,
        probability * 100.0
      );
    }

    self.encode_process(&probabilities, &mut reader, output_filename)?;
    Ok(())
  }

  pub fn decode(&self, _filename: PathBuf) -> () {
    unimplemented!()
  }
}

trait ProbabilityExt {
  fn cumulative(&self) -> f64;
  fn lower_bound(&self) -> f64;
}

impl ProbabilityExt for HashMap<u8, f64> {
  fn cumulative(&self) -> f64 {
    self.values().sum()
  }

  fn lower_bound(&self) -> f64 {
    *self
      .values()
      .min_by(|a, b| a.partial_cmp(b).unwrap())
      .unwrap_or(&0.0)
  }
}
