use aurora;

use metadata;

pub struct Demuxer {
  source: aurora::channel::Source<aurora::Binary>,
  sink: aurora::channel::Sink<aurora::Binary>,
  metadata_sink: aurora::channel::Sink<metadata::Metadata>
}

impl Demuxer {
  pub fn new(source: aurora::channel::Source<aurora::Binary>, sink: aurora::channel::Sink<aurora::Binary>, metadata_sink: aurora::channel::Sink<metadata::Metadata>) -> Demuxer {
    return Demuxer {
      source: source,
      sink: sink,
      metadata_sink: metadata_sink
    }
  }

  pub fn run(&mut self) {
    let mut stream = aurora::stream::Stream::new(&mut self.source);

    let mut fourcc = [0x00, ..4];

    stream.read(fourcc);

    if fourcc != b"fLaC" {
      fail!("flac::Demuxer: Stream did not start with fourcc 'fLaC' had bytes {:x}{:x}{:x}{:x} (INPUT)", fourcc[0], fourcc[1], fourcc[2], fourcc[3]);
    }

    let stream_info_type = stream.read_u8();

    if stream_info_type & 0x7F != 0 {
      fail!("flac::Demuxer: First METADATA_BLOCK was not STREAMINFO (INPUT)");
    }

    let mut last = false;

    while !last {
      self.metadata_sink.write(|metadata| {
        last = metadata::transfer(&mut stream, metadata);
      });
    }

    last = true; // TODO: Actually write data;

    let sink = &mut self.sink;

    while !last {
      sink.write(|_| {
      });
    }
  }
}

#[cfg(all(test, feature = "complete-tests"))]
mod complete_tests {
  use std;
  use aurora;
  use metadata;

  #[test]
  fn test_new() {
    let (sink_0, source_0) = aurora::channel::create::<aurora::Binary>(1);
    let (sink_1, _) = aurora::channel::create::<aurora::Binary>(1);
    let (sink_md, mut source_md) = aurora::channel::create::<metadata::Metadata>(1);

    spawn(proc() {
      let path = std::path::Path::new("./test-vectors/complete/bad_apple.flac");
      let file = std::io::File::open(&path).unwrap();

      aurora::file::Input::new(file, 4096, sink_0).run();
    });

    spawn(proc() {
      super::Demuxer::new(source_0, sink_1, sink_md).run();
    });

    source_md.read(|metadata| {
      let canonical = &metadata::stream_info::StreamInfo {
        block_size: (4096, 4096),
        frame_size: (1324, 13848),
        sample_rate: 44100,
        channels: 2,
        bits_per_sample: 16,
        samples: 13940634,
        signature: metadata::stream_info::MD5([0x07, 0x02, 0x55, 0xE5, 0xCE, 0x94, 0x69, 0xED, 0xC6, 0x23, 0xCD, 0x9E, 0x8E, 0xB3, 0xE2, 0x21])
      };

      assert_eq!(stream_info.ty, metadata::StreamInfo(canonical));
    });

    // let audio = flac.audioStream();
    // 
    // assert_eq!(audio.read_be_u16(), 0xFFF8)
  }
}