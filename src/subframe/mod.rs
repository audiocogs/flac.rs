use std;
use aurora;

#[deriving(Show,PartialEq)]
enum Ty {
  Constant, Verbatim, Fixed(u8), LPC(u8)
}

#[deriving(Show,PartialEq)]
pub struct Header {
  ty: Ty, wasted_bits: u8
}

impl Header {
  pub fn from(stream: &mut aurora::stream::Bitstream) -> Header {
    assert_eq!(stream.read_n(1), 0);

    let ty_code = stream.read_n(6);

    let ty = if ty_code & 0b100000u32 != 0 {
      LPC((ty_code as u8 & 0b011111u8) + 1)
    } else if ty_code & 0b001000u32 != 0 {
      Fixed(ty_code as u8 & 0b000111u8)
    } else if ty_code == 0b000001u32 {
      Verbatim
    } else if ty_code == 0b000000u32 {
      Constant
    } else {
      fail!("flac::Decoder: Value is reserved");
    };

    let wasted = if stream.read_n(1) == 1 {
      let mut n = 1u8;
      while stream.read_n(1) == 0 {
        n += 1;
      }
      n
    } else {
      0
    };

    return Header {
      ty: ty,
      wasted_bits: wasted
    };
  }
}

fn decode_residuals(frame_header: &::frame::header::Header, stream: &mut aurora::stream::Bitstream) -> Vec<u32> {
  let residual_coding_method = stream.read_n(2);

  if residual_coding_method > 1 {
    fail!("Non-compliant residual coding method {}", residual_coding_method);
  }

  let partition_order = stream.read_n(4) as uint;

  let n_samples = frame_header.block_size as uint >> partition_order;

  if partition_order > n_samples {
    fail!("Invalid predictor order");
  }

  let mut subblocks = Vec::new();

  for partition in range(0u, 1 << partition_order) {
    let n_bits = if residual_coding_method == 0 { 4 } else { 5 };
    let tmp = stream.read_n(n_bits) as uint;

    let escape_code = if residual_coding_method == 0 { 0b1111 } else { 0b11111 };
    if tmp == escape_code {
      let tmp = stream.read_n(5) as uint;

      for i in range(0, n_samples) {
        if partition == 0 && i < partition_order {
          subblocks.push(0);
        } else {
          subblocks.push(stream.read_n_signed(tmp) as u32);
        }
      }
    } else {
      for i in range(0, n_samples) {
        if partition == 0 && i < partition_order {
          subblocks.push(0);
        } else {
          subblocks.push(decode_golomb(stream, tmp));
        }
      }
    }
  }

  return subblocks;
}

fn decode_golomb(stream: &mut aurora::stream::Bitstream, parameter: uint) -> u32 {
  let mut msbs = 0;

  loop {
    if stream.read_n(1) == 0b1 {
      break;
    }

    msbs += 1
  }
  
  let lsbs = stream.read_n(parameter);
  
  let uval = ((msbs << parameter) | lsbs) >> 1;
  
  if uval & 1 == 0 {
    return -(uval as i32) as u32 - 1 ;
  } else {
    return uval;
  }
}

#[deriving(Show,PartialEq)]
pub struct LPCSubframe {
  warmup: Vec<uint>,
  precision: u8,
  shift: i8,
  coefficients: Vec<i32>
}

impl LPCSubframe {
  pub fn from(frame_header: &::frame::header::Header, subframe_header: &Header, stream: &mut aurora::stream::Bitstream) -> Vec<u32> {
    let bits_per_sample = frame_header.sample_size;
    let order = match subframe_header.ty {
      LPC(n) => n,
      _ => fail!("Cannot extract order from non LPC subframe")
    };

    let mut subblocks = Vec::new();

    for _ in range(0, order) {
      subblocks.push(stream.read_n(bits_per_sample as uint) as u32);
    }

    let precision = stream.read_n(4) as u8;

    let shift = stream.read_n_signed(5) as i8;

    let mut coefficients = Vec::new();

    for _ in range(0, order) {
      coefficients.push(stream.read_n_signed(precision as uint) as i32);
    }

    let residuals = decode_residuals(frame_header, stream);

    for i in range(order as uint, frame_header.block_size as uint) {
      let mut sum = 0i32;

      for j in range(0, order as uint) {
        sum += coefficients[j] * (residuals[i - j - 1] as i32);
      }

      subblocks.push(sum as u32);
    }

    return subblocks;
  }
}

#[deriving(Show,PartialEq)]
pub struct VerbatimSubframe {
  subblocks: Vec<u32>
}

impl VerbatimSubframe {
  pub fn from(frame_header: &::frame::header::Header, stream: &mut aurora::stream::Bitstream) -> Vec<u32> {
    let bits_per_sample = frame_header.sample_size;
    let block_size = frame_header.block_size;

    let mut subblocks = Vec::new();

    for _ in range(0, block_size) {
      subblocks.push(stream.read_n(bits_per_sample as uint));
    }

    return subblocks;
  }
}

pub fn read(frame_header: &::frame::header::Header, bitstream: &mut aurora::stream::Bitstream) -> Vec<u32> {
  let header = Header::from(bitstream);

  match header.ty {
    Verbatim => {
      return VerbatimSubframe::from(frame_header, bitstream);
    },
    LPC(_) => {
      return LPCSubframe::from(frame_header, &header, bitstream);
    }
    _ => {
      fail!("Unsupported");
    }
  }
}

#[test]
fn test_header_from_1() {
  let (sink_0, mut source_0) = aurora::channel::create::<aurora::Binary>(1);

  spawn(proc() {
    let path = std::path::Path::new("./test-vectors/subframes/bad_apple.1");
    let file = std::io::File::open(&path).unwrap();

    aurora::file::Input::new(file, 4096, sink_0).run();
  });

  let mut stream = aurora::stream::Stream::new(&mut source_0);
  let mut bitstream = aurora::stream::Bitstream::new(&mut stream);

  let header = Header::from(&mut bitstream);

  assert_eq!(header.ty, LPC(1));
  assert_eq!(header.wasted_bits, 0);
}

#[test]
fn test_header_from_2() {
  let (sink_0, mut source_0) = aurora::channel::create::<aurora::Binary>(1);

  spawn(proc() {
    let path = std::path::Path::new("./test-vectors/subframes/bad_apple.2");
    let file = std::io::File::open(&path).unwrap();

    aurora::file::Input::new(file, 4096, sink_0).run();
  });

  let mut stream = aurora::stream::Stream::new(&mut source_0);
  let mut bitstream = aurora::stream::Bitstream::new(&mut stream);

  let header = Header::from(&mut bitstream);

  assert_eq!(header.ty, LPC(1));
  assert_eq!(header.wasted_bits, 0);
}

#[test]
fn test_header_from_3() {
  let (sink_0, mut source_0) = aurora::channel::create::<aurora::Binary>(1);

  spawn(proc() {
    let path = std::path::Path::new("./test-vectors/subframes/bad_apple.3");
    let file = std::io::File::open(&path).unwrap();

    aurora::file::Input::new(file, 4096, sink_0).run();
  });

  let mut stream = aurora::stream::Stream::new(&mut source_0);
  let mut bitstream = aurora::stream::Bitstream::new(&mut stream);

  let header = Header::from(&mut bitstream);

  assert_eq!(header.ty, LPC(1));
  assert_eq!(header.wasted_bits, 0);
}

#[test]
fn test_header_from_bad_apple_verbatim_1() {
  let (sink_0, mut source_0) = aurora::channel::create::<aurora::Binary>(1);
  let (sink_r, mut source_r) = aurora::channel::create::<aurora::Binary>(1);

  spawn(proc() {
    let path = std::path::Path::new("./test-vectors/subframes/bad_apple_verbatim.1");
    let file = std::io::File::open(&path).unwrap();

    aurora::file::Input::new(file, 4096, sink_0).run();
  });

  let mut stream = aurora::stream::Stream::new(&mut source_0);
  let mut bitstream = aurora::stream::Bitstream::new(&mut stream);

  let header = ::frame::header::Header {
    variable_blocksize: false,
    block_size: 1152,
    sample_rate: 44100,
    channel_assignment: 1,
    sample_size: 16,
    sample_number: None,
    frame_number: Some(0),
    crc: 0xAE
  };

  let subframe_header = Header::from(&mut bitstream);

  assert_eq!(subframe_header.ty, Verbatim);
  assert_eq!(subframe_header.wasted_bits, 0);

  let subframe = VerbatimSubframe::from(&header, &mut bitstream);

  spawn(proc() {
    let decoded_path = std::path::Path::new("./test-vectors/subframes/bad_apple_verbatim.1.decoded");
    let decoded_file = std::io::File::open(&decoded_path).unwrap();

    aurora::file::Input::new(decoded_file, 4096, sink_r).run();
  });

  let mut reference_stream = aurora::stream::Stream::new(&mut source_r);

  for i in range(0, 1152) {
    let reference_sample = reference_stream.read_be_u16();
    let decoded_sample = subframe.subblocks[i];

    assert_eq!(decoded_sample, reference_sample as u32);
  }

}
