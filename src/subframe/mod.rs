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