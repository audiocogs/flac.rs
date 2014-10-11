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

impl StreamInfo {
  pub fn transfer(stream: &mut aurora::stream::Stream, result: &mut StreamInfo) {
    let length = stream.read_be_uint_n(3);

    if length != 34 {
      fail!("StreamInfo: Length of block isn't 34, which it should be (INPUT)");
    }

    result.block_size = (stream.read_be_u16(), stream.read_be_u16());
    result.frame_size = (stream.read_be_uint_n(3) as u32, stream.read_be_uint_n(3) as u32);

    let ex = stream.read_be_u64();

    result.sample_rate =     ((ex & 0xFFFFF00000000000) >> 44) as u32;
    result.channels =        ((ex & 0x00000E0000000000) >> 41) as u8 + 1;
    result.bits_per_sample = ((ex & 0x000001F000000000) >> 36) as u8 + 1;
    result.samples =          (ex & 0x0000000FFFFFFFFF) >> 0;

    let mut sig = [0x00u8, ..16];

    stream.read(&mut sig);

    result.signature = MD5(sig);
  }
}

impl aurora::Initialize for StreamInfo {
  fn initialize() -> StreamInfo {
    return StreamInfo {
      block_size: (0, 0),
      frame_size: (0, 0),
      sample_rate: 0,
      channels: 0,
      bits_per_sample: 0,
      samples: 0,
      signature: MD5([0, ..16])
    }
  }

  fn reinitialize(&mut self) {
    self.block_size = (0, 0);
    self.frame_size = (0, 0);
    self.sample_rate = 0;
    self.channels = 0;
    self.bits_per_sample = 0;
    self.samples = 0;
    self.signature = MD5([0, ..16]);
  }
}

#[cfg(test)]
mod tests {
  use std;
  use aurora;

  #[test]
  fn test_from() {
    let (sink_0, mut source_0) = aurora::channel::create::<aurora::Binary>(1);

    spawn(proc() {
      let path = std::path::Path::new("./test-vectors/metadata/bad_apple.stream_info");
      let file = std::io::File::open(&path).unwrap();

      aurora::file::Input::new(file, 4096, sink_0).run();
    });
    
    let mut stream = aurora::stream::Stream::new(&mut source_0);

    let mut stream_info: super::StreamInfo = aurora::Initialize::initialize();
    
    super::StreamInfo::transfer(&mut stream, &mut stream_info);

    let canonical = super::StreamInfo {
      block_size: (4096, 4096),
      frame_size: (1324, 13848),
      sample_rate: 44100,
      channels: 2,
      bits_per_sample: 16,
      samples: 13940634,
      signature: super::MD5([0x07, 0x02, 0x55, 0xE5, 0xCE, 0x94, 0x69, 0xED, 0xC6, 0x23, 0xCD, 0x9E, 0x8E, 0xB3, 0xE2, 0x21])
    };

    assert_eq!(stream_info, canonical);
  }
}