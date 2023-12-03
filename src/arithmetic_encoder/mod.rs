use std::{
  collections::HashMap,
  fs::File,
  io::{BufReader, Read, Result, Seek, SeekFrom},
  path::PathBuf,
};

const PRECISION: f64 = 10000.0;

pub struct ArithmeticEncoder {
  lower_bound: u32,
  upper_bound: u32,
}

impl ArithmeticEncoder {
  pub fn new(lower_bound: u32, upper_bound: u32) -> ArithmeticEncoder {
    ArithmeticEncoder {
      lower_bound,
      upper_bound,
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

    // Sort the vector in descending order by probability
    probabilities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    Ok(probabilities)
  }

  fn encode_process(
    &self,
    sorted_probabilities: &Vec<(u8, f64)>,
    reader: &mut BufReader<File>,
  ) -> Result<u32> {
    // Compute cumulative probabilities
    let mut cumulative_probabilities = HashMap::new();
    let mut cumulative = 0.0;

    for (byte, probability) in sorted_probabilities.into_iter() {
      cumulative += probability;
      cumulative_probabilities.insert(byte, cumulative);
    }

    let mut low = self.lower_bound;
    let mut high = self.upper_bound;
    // let mut range;

    reader.seek(SeekFrom::Start(0))?;

    println!("====================");
    // Read the file byte by byte
    for byte_result in reader.bytes() {
      // find the byte in the cumulative probabilities
      let byte = byte_result?;
      let byte_high_prob = (cumulative_probabilities.get(&byte).unwrap() * PRECISION.trunc()) as u32;
      let byte_low_prob = 0u32;

      println!(
        "Byte: {:?}, Low: {}, High: {}",
        byte as char, byte_low_prob, byte_high_prob
      );

      // let most_significant_hight = ArithmeticEncoder::first_digit(byte_high);
      // let most_significant_low = ArithmeticEncoder::first_digit(byte_low);

      // println!(
      //   "hight: {} - MSH: {} | low: {} - MSL: {}",
      //   byte_high, most_significant_hight, byte_low, most_significant_low
      // );

      // if most_significant_hight == most_significant_low {
      //   // shift the most significant digit out
      //   high = (high << 1) & 0xFFFF_FFFF;
      //   low = (low << 1) & 0xFFFF_FFFF;
      // } else {
      // let new_hight = low + (high - low + 1) * byte_high - 1;
      // let new_low = low + (high - low + 1) * byte_low;
      // }

      // low = new_low;
      // high = new_hight;

      println!("Low: {}, High: {}", low, high);
    }
    println!("====================");

    // Return the midpoint of the final range as the result
    Ok((low + (high - low) / 2) as u32)
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
