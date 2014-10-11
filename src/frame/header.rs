use std;
use aurora;

const SYNC_CODE: u16 = 0b11111111111110;

#[deriving(Show)]
pub struct Header {
  pub variable_blocksize: bool,
  pub block_size: u32,
  pub sample_rate: u32,
  pub channel_assignment: u8,
  pub sample_size: u8,
  pub sample_number: Option<u64>,
  pub frame_number: Option<u32>,
  pub crc: u8
}

impl Header {

  pub fn from(stream: &mut aurora::stream::Bitstream) -> Header {
    if stream.read_n(14) as u16 != SYNC_CODE {
      fail!("Failed to sync frame");
    }

    if stream.read_n(1) != 0 {
      fail!("Reserved bit in frame header must be 0");
    }

    let variable_blocksize = stream.read_n(1) != 0;

    let block_size_code = stream.read_n(4) as u8;

    let sample_rate_code = stream.read_n(4) as u8;

    let channel_assignment = stream.read_n(4) as u8;

    let sample_size = Header::finalize_sample_size(stream.read_n(3) as u8);

    if stream.read_n(1) != 0 {
      fail!("Reserved bit in frame header must be 0");
    }

    let decoded_number = decode_sample_or_frame_number(stream);

    let mut frame_number: Option<u32> = None;
    let mut sample_number: Option<u64> = None;

    if variable_blocksize {
      sample_number = Some(decoded_number);
    } else {
      frame_number = Some(decoded_number as u32);
    }

    let block_size = Header::finalize_block_size(block_size_code, stream);

    let sample_rate = Header::finalize_sample_rate(sample_rate_code, stream);

    let crc = stream.read_n(8) as u8;

    return Header {
      variable_blocksize: variable_blocksize,
      block_size: block_size,
      sample_rate: sample_rate,
      channel_assignment: channel_assignment,
      sample_size: sample_size,
      sample_number: sample_number,
      frame_number: frame_number,
      crc: crc
    };
  }

  fn finalize_block_size(block_size_code: u8, stream: &mut aurora::stream::Bitstream) -> u32 {
    let n = block_size_code as uint;

    return match n {
      0b0000 => fail!("Block size 0000 is reserved"),
      0b0001 => 192,
      0b0010 => 576 << (n - 2),
      0b0011 => 576 << (n - 2),
      0b0100 => 576 << (n - 2),
      0b0101 => 576 << (n - 2),
      0b0110 => stream.read_n(8) + 1,
      0b0111 => stream.read_n(16) + 1,
      0b1000 => 256 << (n - 8),
      0b1001 => 256 << (n - 8),
      0b1010 => 256 << (n - 8),
      0b1011 => 256 << (n - 8),
      0b1100 => 256 << (n - 8),
      0b1101 => 256 << (n - 8),
      0b1110 => 256 << (n - 8),
      0b1111 => 256 << (n - 8),
      _ => fail!("Invalid block size")
    };
  }

  fn finalize_sample_rate(sample_rate_code: u8, stream: &mut aurora::stream::Bitstream) -> u32 {
    match sample_rate_code {
      0b0000 => fail!("TODO: get from STREAMINFO metadata block"),
      0b0001 => 88_200,
      0b0010 => 176_400,
      0b0011 => 192_000,
      0b0100 => 8_000,
      0b0101 => 16_000,
      0b0110 => 22_050,
      0b0111 => 24_000,
      0b1000 => 32_000,
      0b1001 => 44_100,
      0b1010 => 48_000,
      0b1011 => 96_000,
      0b1100 => stream.read_n(8) * 1000,
      0b1101 => stream.read_n(16),
      0b1110 => stream.read_n(16) * 10,
      _ => fail!("Invalid sample rate")
    }
  }

  fn finalize_sample_size(sample_size_code: u8) -> u8 {
    match sample_size_code {
      0b000 => fail!("TODO: get from STREAMINFO metadata block"),
      0b001 => 8,
      0b010 => 12,
      0b011 => fail!("flac::Decoder: Reserved sample size (INPUT)"),
      0b100 => 16,
      0b101 => 20,
      0b110 => 24,
      0b111 => fail!("flac::Decoder: Reserved sample size (INPUT)"),
      _ => fail!("flac::Decoder: Undefined input?! (BUG)")
    }
  }

}

// See http://en.wikipedia.org/wiki/UTF-8
fn decode_sample_or_frame_number(stream: &mut aurora::stream::Bitstream) -> u64 {
  let mut total_bytes = 0;

  while stream.read_n(1) == 1 {
    total_bytes += 1;
  }

  let mut decoded = stream.read_n(7 - total_bytes) as u64;

  for _ in range(1, total_bytes) {
    assert_eq!(stream.read_n(2), 0b10u32);
    decoded = (decoded << 6) + stream.read_n(6) as u64;
  }

  return decoded;
}

#[test]
fn test_utf8_decoding_of_one_byte() {
  let (sink_0, mut source_0) = aurora::channel::create::<aurora::Binary>(1);

  spawn(proc() {
    let buffer = vec![0b00100100];
    aurora::buffer::Buffer::new(buffer, 4096, sink_0).run();
  });

  let mut stream = aurora::stream::Stream::new(&mut source_0);
  let mut bitstream = aurora::stream::Bitstream::new(&mut stream);

  let decoded = decode_sample_or_frame_number(&mut bitstream);

  assert_eq!(decoded, 0b0100100);
}

#[test]
fn test_utf8_decoding_of_four_bytes() {
  let (sink_0, mut source_0) = aurora::channel::create::<aurora::Binary>(1);

  spawn(proc() {
    let buffer = vec![0b11110000, 0b10100100, 0b10101101, 0b10100010];
    aurora::buffer::Buffer::new(buffer, 4096, sink_0).run();
  });

  let mut stream = aurora::stream::Stream::new(&mut source_0);
  let mut bitstream = aurora::stream::Bitstream::new(&mut stream);

  let decoded = decode_sample_or_frame_number(&mut bitstream);

  assert_eq!(decoded, 0b000100100101101100010);
}

#[test]
fn test_header_from_1() {
  let (sink_0, mut source_0) = aurora::channel::create::<aurora::Binary>(1);

  spawn(proc() {
    let path = std::path::Path::new("./test-vectors/frames/bad_apple.1");
    let file = std::io::File::open(&path).unwrap();

    aurora::file::Input::new(file, 4096, sink_0).run();
  });

  let mut stream = aurora::stream::Stream::new(&mut source_0);
  let mut bitstream = aurora::stream::Bitstream::new(&mut stream);

  let header = Header::from(&mut bitstream);

  assert_eq!(header.variable_blocksize, false);
  assert_eq!(header.block_size, 4096);
  assert_eq!(header.sample_rate, 44100);
  assert_eq!(header.channel_assignment, 1);
  assert_eq!(header.sample_size, 16);
  assert_eq!(header.frame_number, Some(0));
  assert_eq!(header.crc, 0xC2);
}

#[test]
fn test_header_from_2() {
  let (sink_0, mut source_0) = aurora::channel::create::<aurora::Binary>(1);

  spawn(proc() {
    let path = std::path::Path::new("./test-vectors/frames/bad_apple.2");
    let file = std::io::File::open(&path).unwrap();

    aurora::file::Input::new(file, 4096, sink_0).run();
  });

  let mut stream = aurora::stream::Stream::new(&mut source_0);
  let mut bitstream = aurora::stream::Bitstream::new(&mut stream);

  let header = Header::from(&mut bitstream);

  assert_eq!(header.variable_blocksize, false);
  assert_eq!(header.block_size, 4096);
  assert_eq!(header.sample_rate, 44100);
  assert_eq!(header.channel_assignment, 1);
  assert_eq!(header.sample_size, 16);
  assert_eq!(header.frame_number, Some(1));
  assert_eq!(header.crc, 0xC5);
}

#[test]
fn test_header_from_3() {
  let (sink_0, mut source_0) = aurora::channel::create::<aurora::Binary>(1);

  spawn(proc() {
    let path = std::path::Path::new("./test-vectors/frames/bad_apple.3");
    let file = std::io::File::open(&path).unwrap();

    aurora::file::Input::new(file, 4096, sink_0).run();
  });

  let mut stream = aurora::stream::Stream::new(&mut source_0);
  let mut bitstream = aurora::stream::Bitstream::new(&mut stream);

  let header = Header::from(&mut bitstream);

  assert_eq!(header.variable_blocksize, false);
  assert_eq!(header.block_size, 4096);
  assert_eq!(header.sample_rate, 44100);
  assert_eq!(header.channel_assignment, 1);
  assert_eq!(header.sample_size, 16);
  assert_eq!(header.frame_number, Some(2));
  assert_eq!(header.crc, 0xCC);
}