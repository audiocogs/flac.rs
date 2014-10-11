use std::fmt;
use std::cmp;

use aurora;

pub struct MD5(pub [u8, ..16]);

impl fmt::Show for MD5 {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let MD5(d) = *self;

    return write!(f, "MD5({:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x}{:x})", d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7], d[8], d[9], d[10], d[11], d[12], d[13], d[14], d[15]);
  }
}

impl cmp::PartialEq for MD5 {
  fn eq(&self, other: &MD5) -> bool {
    let MD5(a) = *self;
    let MD5(b) = *other;

    for i in range(0u, 16) {
      if a[i] != b[i] {
        return false;
      }
    }

    return true;
  }
}

#[deriving(Show,PartialEq)]
pub struct StreamInfo {
  pub block_size: (u16, u16),
  pub frame_size: (u32, u32),
  pub sample_rate: u32,
  pub channels: u8,
  pub bits_per_sample: u8,
  pub samples: u64,
  pub signature: MD5
}

pub fn read(data: &Vec<u8>) -> StreamInfo {
  if data.len() != 34 {
    fail!("StreamInfo: Length of block isn't 37, which it should be (INPUT)");
  }

  let data = data.as_slice().to_vec();

  let (sink, mut source) = aurora::channel::create::<aurora::Binary>(1);

  spawn(proc() {
    aurora::buffer::Buffer::new(data, 4096, sink).run();
  });

  let mut stream = aurora::stream::Stream::new(&mut source);

  let block_size = (stream.read_be_u16(), stream.read_be_u16());
  let frame_size = (stream.read_be_uint_n(3) as u32, stream.read_be_uint_n(3) as u32);

  let ex = stream.read_be_u64();

  let sample_rate =     ((ex & 0xFFFFF00000000000) >> 44) as u32;
  let channels =        ((ex & 0x00000E0000000000) >> 41) as u8 + 1;
  let bits_per_sample = ((ex & 0x000001F000000000) >> 36) as u8 + 1;
  let samples =          (ex & 0x0000000FFFFFFFFF) >> 0;

  let mut sig = [0x00u8, ..16];

  stream.read(&mut sig);

  let signature = MD5(sig);

  return StreamInfo {
    block_size: block_size,
    frame_size: frame_size,
    sample_rate: sample_rate,
    channels: channels,
    bits_per_sample: bits_per_sample,
    samples: samples,
    signature: signature
  };
}
