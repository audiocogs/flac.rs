use aurora;

use metadata;

pub trait FlacDemuxer<'a> {
  fn audioStream(&'a mut self) -> &'a mut aurora::stream::Stream;
}

pub struct FlacDemuxerUnseekable<'a> {
    stream: &'a mut aurora::stream::Stream + 'a,
    stream_info: metadata::stream_info::StreamInfo
}

impl<'a> FlacDemuxerUnseekable<'a> {
  pub fn new(stream: &'a mut aurora::stream::Stream) -> FlacDemuxerUnseekable {
    let mut fourcc = [0x00, ..4];

    stream.read(fourcc);

    if (fourcc != b"fLaC") {
      fail!("FlacDemuxerUnseekable: Stream did not start with fourcc 'fLaC' (INPUT)");
    }

    let stream_info_type = stream.read_u8();

    if stream_info_type & 0x7F != 0 {
      fail!("FlacDemuxerUnseekable: First METADATA_BLOCK was not STREAMINFO (INPUT)");
    }

    let stream_info = metadata::stream_info::StreamInfo::from(stream);

    let mut last = stream_info_type & 0x80 != 0;

    while (!last) {
      let block_type = stream.read_u8();

      last = block_type & 0x80 != 0;

      match block_type & 0x7F {
        0 => {
          fail!("FlacDemuxerUnseekable: Multiple STREAMINFO (INPUT)")
        },
        n if n < 127 => {
          let _ = metadata::unknown::Unknown::from(stream);
        }
        n => {
          fail!("FlacDemuxerUnseekable: METADATA_BLOCK BLOCK_TYPE is {}, which is invalid (INPUT)", n)
        }
      }
    }

    return FlacDemuxerUnseekable { stream: stream, stream_info: stream_info };
  }
}

impl<'a> FlacDemuxer<'a> for FlacDemuxerUnseekable<'a> {
  fn audioStream(&'a mut self) -> &'a mut aurora::stream::Stream {
    return self.stream;
  }
}

#[cfg(test)]
mod tests {
  use std;
  use aurora;
  use metadata;
  
  use super::FlacDemuxer;

  #[test]
  #[cfg(feature = "complete-tests")]
  fn test_new() {
    let path = std::path::Path::new("./test-vectors/complete/bad_apple.flac");
    let mut s = aurora::file::FileStream::new(std::io::File::open(&path).unwrap());

    let mut flac = super::FlacDemuxerUnseekable::new(&mut s);

    let canonical = metadata::stream_info::StreamInfo {
      block_size: (4096, 4096),
      frame_size: (1324, 13848),
      sample_rate: 44100,
      channels: 2,
      bits_per_sample: 16,
      samples: 13940634,
      signature: metadata::stream_info::MD5([0x07, 0x02, 0x55, 0xE5, 0xCE, 0x94, 0x69, 0xED, 0xC6, 0x23, 0xCD, 0x9E, 0x8E, 0xB3, 0xE2, 0x21])
    };

    assert_eq!(flac.stream_info, canonical);

    let audio = flac.audioStream();

    assert_eq!(audio.read_be_u16(), 0xFFF8)
  }
  
}