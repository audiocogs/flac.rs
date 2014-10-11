use aurora;

pub struct Unknown {
  data: Vec<u8>
}

impl Unknown {
  pub fn from(stream: &mut aurora::stream::Stream) -> Unknown {
    let length = stream.read_be_uint_n(3);

    let mut data = Vec::from_elem(length as uint, 0x00u8);

    stream.read(data.as_mut_slice());

    return Unknown { data: data };
  }
}

#[cfg(test)]
mod tests {
  use std;
  use aurora;

  #[test]
  fn test_from_1() {
    let (sink_0, mut source_0) = aurora::channel::create::<aurora::Binary>(1);

    spawn(proc() {
      let path = std::path::Path::new("./test-vectors/metadata/bad_apple.stream_info");
      let file = std::io::File::open(&path).unwrap();

      aurora::file::Input::new(file, 4096, sink_0).run();
    });
    
    let mut stream = aurora::stream::Stream::new(&mut source_0);

    let si = super::Unknown::from(&mut stream);

    assert_eq!(si.data.len(), 34);
  }

  #[test]
  fn test_from_2() {
    let (sink_0, mut source_0) = aurora::channel::create::<aurora::Binary>(1);

    spawn(proc() {
      let path = std::path::Path::new("./test-vectors/metadata/bad_apple.vorbis_comment");
      let file = std::io::File::open(&path).unwrap();

      aurora::file::Input::new(file, 4096, sink_0).run();
    });
    
    let mut stream = aurora::stream::Stream::new(&mut source_0);

    let si = super::Unknown::from(&mut stream);

    assert_eq!(si.data.len(), 315);
  }
}