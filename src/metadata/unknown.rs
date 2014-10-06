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
    let path = std::path::Path::new("./test-vectors/metadata/bad_apple.stream_info");
    let mut s = aurora::file::FileStream::new(std::io::File::open(&path).unwrap());

    let si = super::Unknown::from(&mut s);

    assert_eq!(si.data.len(), 34);
  }

  #[test]
  fn test_from_2() {
    let path = std::path::Path::new("./test-vectors/metadata/bad_apple.vorbis_comment");
    let mut s = aurora::file::FileStream::new(std::io::File::open(&path).unwrap());

    let si = super::Unknown::from(&mut s);

    assert_eq!(si.data.len(), 315);
  }
}