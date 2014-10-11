use std;

use aurora;

pub struct Decoder {
  source: aurora::channel::Source<aurora::Binary>,
  metadata_source: aurora::channel::Source<::metadata::Metadata>,
  sink: aurora::channel::Sink<aurora::Audio>
}

impl Decoder {
  pub fn new(source: aurora::channel::Source<aurora::Binary>, metadata_source: aurora::channel::Source<::metadata::Metadata>, sink: aurora::channel::Sink<aurora::Audio>) -> Decoder {
    return Decoder { source: source, metadata_source: metadata_source, sink: sink };
  }

  pub fn run(&mut self) {
    let mut ty = ::metadata::Unknown;
    self.metadata_source.read(|metadata| { ty = metadata.ty });

    let stream_info = match ty {
      ::metadata::StreamInfo(si) => si,
      _ => fail!("Metadata didn't start with a stream info, and it has to according to the spec")
    };

    let mut last = false;
    let mut samples_remaining = stream_info.samples;

    let mut stream = aurora::stream::Stream::new(&mut self.source);
    let mut bitstream = aurora::stream::Bitstream::new(&mut stream);

    while !last {
      let bs = &mut bitstream;
      let sink = &mut self.sink;
      
      sink.write(|audio| {
        let samples = ::frame::read(bs, audio);

        samples_remaining -= samples as u64;
        
        last = samples_remaining == 0;
        
        audio.last = last;
      });
    }
  }
}
