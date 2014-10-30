use std;
use std::mem;

use aurora;

pub mod header;

fn extend_sign_bits(value: u32, n: u8) -> i32 {
  let shift = 32 - n;

  return (value << shift as uint) as i32 >> (shift as uint);
}

pub fn read(bitstream: &mut aurora::stream::Bitstream, audio: &mut aurora::Audio) -> uint {
  let header = header::Header::from(bitstream);

  let channels = match header.channel_assignment {
    1 => 2,
    _ => panic!("Not implemented")
  };

  let mut subframes = Vec::new();
  for _ in range(0, channels) {
    subframes.push(super::subframe::read(&header, bitstream))
  }

  match header.channel_assignment {
    1 => {
      audio.channels = channels;
      audio.sample_rate = header.sample_rate as f64;
      audio.endian = aurora::endian::Big;
      audio.sample_type = aurora::sample_type::Signed(header.sample_size as uint);

      let bytes_per_sample = header.sample_size as uint / 8;

      audio.data.grow(bytes_per_sample * channels * header.block_size as uint, 0);

      for s in range(0, header.block_size as uint) {
        for c in range(0, channels) {
          let sample = unsafe { mem::transmute::<i32, [u8, ..4]>(extend_sign_bits(subframes[c][s], header.sample_size).to_be()) };

          let index = bytes_per_sample * (s * channels + c);

          let input = sample.slice(4 - bytes_per_sample, 4);
          let output =  audio.data.slice_mut(index, index + bytes_per_sample);

          std::slice::bytes::copy_memory(output, input);
        }
      }
    },
    _ => panic!("Not implemented")
  }

  let _ = bitstream.read_n(16); // CRC

  return header.block_size as uint;
}