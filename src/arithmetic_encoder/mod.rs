use std::{
  cmp::Ordering::Equal,
  collections::HashMap,
  fs::File,
  io::{BufReader, Read, Result, Seek, SeekFrom, Write},
  path::PathBuf,
};

const _BUFFERING_SIZE: u16 = 10;

pub struct ArithmeticEncoder {
  lower_bound: u32,
  upper_bound: u32,
  probabilities: Vec<(u8, f64)>,
}

impl ArithmeticEncoder {
  pub fn new(lower_bound: u32, upper_bound: u32) -> ArithmeticEncoder {
    ArithmeticEncoder {
      lower_bound,
      upper_bound,
      probabilities: Vec::new(),
    }
  }

  fn _remove_trailing_zeros(&self, number: u32) -> u32 {
    let binding = number.to_string();
    let trimmed_str = binding.trim_end_matches('0');
    if trimmed_str.is_empty() {
      // If all digits were zeros, return 0
      0
    } else {
      trimmed_str.parse::<u32>().unwrap()
    }
  }

  fn _get_minimal_integer(&self, mut number: u32) -> Vec<u8> {
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

  fn first_digit(number: u32) -> u8 {
    number
      .to_string()
      .chars()
      .next()
      .unwrap()
      .to_digit(10)
      .unwrap() as u8
  }

  fn calculate_probabilities(&mut self, reader: &mut BufReader<File>) -> Result<()> {
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
    self.probabilities = frequency_map
      .into_iter()
      .map(|(byte, count)| (byte, count as f64 / total_bytes as f64))
      .collect();

    // Sort the vector in descending order by probability. if probabilities are equal, sort by byte value
    self.probabilities.sort_by(|a, b| {
      b.1
        .partial_cmp(&a.1)
        .unwrap_or(Equal)
        .then_with(|| a.0.cmp(&b.0))
    });

    Ok(())
  }
  // TODO: Find a way to do a buffer to write to the file
  fn encode_process(
    &self,
    sorted_probabilities: &Vec<(u8, f64)>,
    reader: &mut BufReader<File>,
    mut output_file: File,
  ) -> Result<()> {
    let mut output_vec: Vec<u8> = Vec::new();

    // Compute cumulative probabilities
    let cumulative_probabilities = sorted_probabilities.compute_cumulative_probabilities();

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

    for byte_result in reader.bytes() {
      let byte = byte_result?;

      let (index, byte_high_prob) = cumulative_probabilities.find_byte_data(byte);
      let byte_low_prob = if index == 0 {
        0.0
      } else {
        cumulative_probabilities[index - 1].1
      };

      let range = (high - low) as f64 + 1.0;
      let prob_in_range_high = (range * byte_high_prob).trunc() as u32;
      let prob_in_range_low = (range * byte_low_prob).trunc() as u32;

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
        output_file.write_all(&[most_significant_high])?;
      }

      low = new_low;
      high = new_high;
    }

    // WARNING: not sure about this (store as le or be)
    output_file.write_all(&low.to_le_bytes())?;
    output_vec.append(&mut low.to_le_bytes().to_vec());

    println!("{:?}", output_vec);

    Ok(())
  }

  fn encode_header(&self, probabilities: &Vec<(u8, f64)>, mut output_file: File) -> Result<()> {
    // write the lower and upper bounds to the output file
    output_file.write_all(&self.lower_bound.to_le_bytes())?;
    output_file.write_all(&self.upper_bound.to_le_bytes())?;

    // write the number of entries at the probabilities table
    output_file.write_all(&(probabilities.len() as u32).to_le_bytes())?;

    // write the probabilities to the output file
    for (byte, probability) in probabilities.clone() {
      output_file.write_all(&[byte])?;
      output_file.write_all(&probability.to_le_bytes())?;
    }

    Ok(())
  }

  pub fn encode(&mut self, filename: PathBuf, output_filename: PathBuf) -> Result<()> {
    // Open the file as a binary file
    let file = File::open(&filename).unwrap();
    let output_file = File::create(&output_filename.with_extension("ac")).unwrap();
    let mut reader = BufReader::new(file);
    self.calculate_probabilities(&mut reader).unwrap();

    for (byte, probability) in self.probabilities.clone() {
      println!(
        "Byte: {:?}, Probability: {:.2}%",
        byte as char,
        probability * 100.0
      );
    }

    self.encode_header(&self.probabilities, output_file.try_clone()?)?;

    self.encode_process(&self.probabilities, &mut reader, output_file)?;
    Ok(())
  }

  fn decode_header(&mut self, mut input_file: File) -> Result<()> {
    // read the lower and upper bounds from the input file
    let mut lower_bound_bytes = [0u8; 4];
    let mut upper_bound_bytes = [0u8; 4];
    input_file.read_exact(&mut lower_bound_bytes)?;
    input_file.read_exact(&mut upper_bound_bytes)?;

    self.lower_bound = u32::from_le_bytes(lower_bound_bytes);
    self.upper_bound = u32::from_le_bytes(upper_bound_bytes);

    // read the number of entries at the probabilities table
    let mut number_of_entries_bytes = [0u8; 4];
    input_file.read_exact(&mut number_of_entries_bytes)?;
    let number_of_entries = u32::from_le_bytes(number_of_entries_bytes);

    // read the probabilities from the input file
    for _ in 0..number_of_entries {
      let mut byte = [0u8; 1];
      let mut probability_bytes = [0u8; 8];
      input_file.read_exact(&mut byte)?;
      input_file.read_exact(&mut probability_bytes)?;
      let probability = f64::from_le_bytes(probability_bytes);
      self.probabilities.push((byte[0], probability));
    }

    Ok(())
  }

  // pub fn decode(&self, encoded_filename: PathBuf, output_filename: PathBuf) -> Result<()> {
  //   let encoded_file = File::open(&encoded_filename)?;
  //   let mut reader = BufReader::new(encoded_file);
  //   let mut output_file = File::create(&output_filename)?;

  //   // Read the header information (lower_bound, upper_bound, probabilities)
  //   self.decode_header(encoded_file.try_clone()?)?;

  //   // Initialize decoding variables
  //   let mut current_range = (self.lower_bound as f64, self.upper_bound as f64);

  //   // Read the encoded data and reconstruct the original data
  //   while let Some(encoded_value) = self.read_next_encoded_value(&mut reader) {
  //     let byte = self.find_corresponding_byte(encoded_value, &probabilities, &mut current_range);
  //     output_file.write_all(&[byte])?;
  //   }

  //   Ok(())
  // }

  pub fn decode(&mut self, filename: PathBuf) -> Result<()> {
    let mut input_file = File::open(&filename).unwrap();
    self.decode_header(input_file.try_clone()?)?;

    // read the last 4 bytes from the file in u32
    let mut last_bytes = [0u8; 4];
    input_file.seek(SeekFrom::End(-4))?;
    input_file.read_exact(&mut last_bytes)?;
    let last_low = u32::from_le_bytes(last_bytes);

    println!("Last bytes: {}", last_low);

    println!("====================");
    println!(
      "Lower bound: {} | Upper bound: {}",
      self.lower_bound, self.upper_bound
    );
    for (byte, probability) in self.probabilities.clone() {
      println!(
        "Byte: {:?}, Probability: {:.2}%",
        byte as char,
        probability * 100.0
      );
    }
    println!("====================");

    let mut input_file_reader = BufReader::new(input_file.try_clone()?);
    // place the cursor at the end of the probabilities table
    input_file_reader.seek(SeekFrom::Start(12 + self.probabilities.len() as u64 * 9))?;

    // read the byte by byte from the file
    for byte in input_file_reader.bytes() {
      let byte = byte?;
      let range = (self.upper_bound - self.lower_bound) as f64 + 1.0;
      let calc = self.lower_bound as f64 + (byte as f64 / u32::MAX as f64) * range;
      println!("Byte: {}: {}", byte, calc);
    }

    Ok(())
  }
}

// TODO: implement it for the current data
trait ProbabilityOperations {
  fn compute_cumulative_probabilities(&self) -> Vec<(u8, f64)>;
  fn find_byte_data(&self, byte: u8) -> (usize, f64);
}

impl ProbabilityOperations for Vec<(u8, f64)> {
  fn compute_cumulative_probabilities(&self) -> Vec<(u8, f64)> {
    let mut cumulative_probabilities = Vec::new();
    let mut cumulative = 0.0;

    for &(byte, probability) in self.iter() {
      cumulative += probability;
      cumulative_probabilities.push((byte, cumulative));
    }

    cumulative_probabilities
  }

  fn find_byte_data(&self, byte: u8) -> (usize, f64) {
    self
      .iter()
      .enumerate()
      .find(|&(_, (b, _))| *b == byte)
      .map(|(index, &(_, prob))| (index, prob))
      .unwrap()
  }
}
