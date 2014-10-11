use aurora;

pub mod stream_info;

#[deriving(Show,PartialEq)]
enum Ty {
  StreamInfo(stream_info::StreamInfo), Unknown
}

#[deriving(Show,PartialEq)]
pub struct Metadata {
  ty: Ty,
  data: Vec<u8>
}

impl aurora::Initialize for Metadata {
  fn initialize() -> Metadata {
    return Metadata { ty: Unknown, data: Vec::with_capacity(4096) }
  }

  fn reinitialize(&mut self) {
    self.ty = Unknown;
    self.data.truncate(0);
  }
}

pub fn transfer(stream: &mut aurora::stream::Stream, result: &mut Metadata) -> bool {
  let header = stream.read_u8();
  let length = stream.read_be_uint_n(3);

  let last = header & 0x80 != 0;
  let ty = header & 0x7F;

  result.data.grow(length as uint, 0x00u8);
  stream.read(result.data.as_mut_slice());

  match ty {
    0 => {
      result.ty = StreamInfo(stream_info::read(&result.data))
    }
    _ => {
      result.ty = Unknown;
    }
  }

  return last;
}

#[cfg(test)]
mod tests {
  use std;
  use aurora;

  #[test]
  fn test_transfer_1() {
    let (sink_0, mut source_0) = aurora::channel::create::<aurora::Binary>(1);

    spawn(proc() {
      let path = std::path::Path::new("./test-vectors/metadata/bad_apple.stream_info");
      let file = std::io::File::open(&path).unwrap();

      aurora::file::Input::new(file, 4096, sink_0).run();
    });

    let mut stream = aurora::stream::Stream::new(&mut source_0);

    let mut metadata = aurora::Initialize::initialize();

    let last = super::transfer(&mut stream, &mut metadata);

    assert_eq!(last, false);

    assert_eq!(metadata.ty, super::StreamInfo(super::stream_info::StreamInfo {
      block_size: (4096, 4096),
      frame_size: (1324, 13848),
      sample_rate: 44100,
      channels: 2,
      bits_per_sample: 16,
      samples: 13940634,
      signature: super::stream_info::MD5([0x07, 0x02, 0x55, 0xE5, 0xCE, 0x94, 0x69, 0xED, 0xC6, 0x23, 0xCD, 0x9E, 0x8E, 0xB3, 0xE2, 0x21])
    }));

    assert_eq!(metadata.data.len(), 34);
  }

  #[test]
  fn test_transfer_2() {
    let (sink_0, mut source_0) = aurora::channel::create::<aurora::Binary>(1);

    spawn(proc() {
      let path = std::path::Path::new("./test-vectors/metadata/bad_apple.vorbis_comment");
      let file = std::io::File::open(&path).unwrap();

      aurora::file::Input::new(file, 4096, sink_0).run();
    });

    let mut stream = aurora::stream::Stream::new(&mut source_0);

    let mut metadata = aurora::Initialize::initialize();

    let last = super::transfer(&mut stream, &mut metadata);

    assert_eq!(last, true);
    assert_eq!(metadata.ty, super::Unknown);
    assert_eq!(metadata.data.len(), 315);
  }
}