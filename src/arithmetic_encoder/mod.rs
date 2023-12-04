use std::{
  collections::HashMap,
  fs::File,
  io::{BufReader, Read, Result, Seek, SeekFrom, Write},
  path::PathBuf,
};

pub struct ArithmeticEncoder {
  lower_bound: u32,
  upper_bound: u32,
  precision: f64,
}

impl ArithmeticEncoder {
  pub fn new(lower_bound: u32, upper_bound: u32) -> ArithmeticEncoder {
    ArithmeticEncoder {
      lower_bound,
      upper_bound,
      precision: 10f64.powi((upper_bound.ilog10() as i32).max(0)) * 10.0,
    }
  }

  fn first_digit(number: u32) -> u32 {
    number
      .to_string()
      .chars()
      .next()
      .unwrap()
      .to_digit(10)
      .unwrap()
  }

  fn trunc_in_precision(&self, number: f64) -> f64 {
    (number * self.precision).trunc() / self.precision
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
      .map(|(byte, count)| {
        (
          byte,
          self.trunc_in_precision(count as f64 / total_bytes as f64),
        )
      })
      .collect();

    // Sort the vector in descending order by probability
    probabilities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    Ok(probabilities)
  }

  fn encode_process(
    &self,
    sorted_probabilities: &Vec<(u8, f64)>,
    reader: &mut BufReader<File>,
  ) -> Result<u32> {
    let mut output = File::create("output.txt")?;
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

    let mut low = self.lower_bound as f64;
    let mut high = self.upper_bound as f64;
    // let mut range;

    reader.seek(SeekFrom::Start(0))?;

    println!("====================");
    // Read the file byte by byte
    for byte_result in reader.bytes() {
      // find the byte in the cumulative probabilities
      let byte = byte_result?;
      let (index, (_, byte_high_prob)) = cumulative_probabilities
        .iter()
        .enumerate()
        .find(|&(_, (b, _))| b == &byte)
        .unwrap();

      let byte_low_prob = if index == 0 {
        0.0
      } else {
        cumulative_probabilities[index - 1].1
      };

      let range = high - low + 1.0;
      let mut new_low: f64 = low + range * byte_low_prob;
      let mut new_high: f64 = low + range * (*byte_high_prob) - 1.0;

      let most_significant_high: u32 = ArithmeticEncoder::first_digit(new_high as u32);
      let most_significant_low: u32 = ArithmeticEncoder::first_digit(new_low as u32);

      if most_significant_high == most_significant_low {
        println!("SHIFTING -> LOW: {} | HIGH: {}", new_low, new_high);

        // Calculate the power of 10 to remove the most significant digit
        let power_of_10 = 10f64.powi((new_high.log10() as i32).max(0));

        // Remove the most significant digit
        new_high = (new_high - most_significant_high as f64 * power_of_10) * 10.0;
        new_low = (new_low - most_significant_low as f64 * power_of_10) * 10.0;

        // write the most significant digit as a string to the output
        output.write_all(most_significant_high.to_string().as_bytes())?;
      }

      low = new_low.round();
      high = new_high.round();

      println!("Char: {}: Low: {}, High: {}", byte as char, low, high);
    }
    println!("====================");

    // Return the midpoint of the final range as the result
    Ok((low + (high - low) / 2.0) as u32)
  }

  pub fn encode(&self, filename: PathBuf) -> () {
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

    let encoded_value = self.encode_process(&probabilities, &mut reader).unwrap();

    println!("Encoded Value: {}", encoded_value);
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
