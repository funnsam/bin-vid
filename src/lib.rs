use std::io;

mod utils;

pub fn encode_video<'a, S: 'a + Iterator<Item = Vec<bool>>>(size: (u16, u16), stream: S) -> Vec<u8> {
    let mut bits = utils::BitsWriter::new();
    bits.write_u16_aligned(size.0);
    bits.write_u16_aligned(size.1);

    let mut last_frame: Option<Vec<bool>> = None;

    for frame in stream {
        assert_eq!(frame.len(), size.0 as usize * size.1 as usize);

        if let Some(last) = last_frame {
            if size_of_p_frame(&last, &frame) < size_of_i_frame(&frame) {
                write_p_frame(&mut bits, &last, &frame);
            } else {
                write_i_frame(&mut bits, &frame);
            }
        } else {
            write_i_frame(&mut bits, &frame);
        }

        last_frame = Some(frame);
    }

    bits.bits
}

fn write_i_frame(bits: &mut utils::BitsWriter, frame: &[bool]) {
    bits.write_bit(false); // I-frame indication
    bits.write_bit(frame[0]);

    let mut last = frame[0];
    let mut count = 0;

    for b in frame.iter() {
        if *b != last {
            bits.write_rle_packet(count);

            count = 1;
            last = *b;
        } else {
            count += 1;
        }
    }

    bits.write_rle_packet(count);
}

fn write_p_frame(bits: &mut utils::BitsWriter, last_f: &[bool], frame: &[bool]) {
    bits.write_bit(true); // P-frame indication
    bits.write_bit(last_f[0] ^ frame[0]);

    let mut last = last_f[0] ^ frame[0];
    let mut count = 0;

    for (l, b) in last_f.iter().zip(frame.iter()) {
        if *l ^ *b != last {
            bits.write_rle_packet(count);

            count = 1;
            last = *l ^ *b;
        } else {
            count += 1;
        }
    }

    bits.write_rle_packet(count);
}

fn size_of_i_frame(frame: &[bool]) -> usize {
    let mut size = 0;
    let mut last = frame[0];
    let mut count = 0_usize;

    for b in frame.iter() {
        if *b != last {
            size += 2 * (count.ilog2() + 1) as usize;

            count = 1;
            last = *b;
        } else {
            count += 1;
        }
    }

    size + 2 * (count.ilog2() + 1) as usize
}

fn size_of_p_frame(last_f: &[bool], frame: &[bool]) -> usize {
    let mut size = 0;
    let mut last = last_f[0] ^ frame[0];
    let mut count = 0_usize;

    for (l, b) in last_f.iter().zip(frame.iter()) {
        if *l ^ *b != last {
            size += 2 * (count.ilog2() + 1) as usize;

            count = 1;
            last = *l ^ *b;
        } else {
            count += 1;
        }
    }

    size + 2 * (count.ilog2() + 1) as usize
}

pub struct Decoder<R: io::Read> {
    reader: io::BufReader<R>,
    last_frame: Option<Vec<u8>>,
    frame_size: u128,
}

impl<R: io::Read> Decoder<R> {
    pub fn new(mut reader: R) -> Result<(Self, (u16, u16)), io::Error> {
        let mut buf = [0; 4];
        reader.read_exact(&mut buf)?;

        let width = buf[0] as u16 | ((buf[1] as u16) << 8);
        let height = buf[2] as u16 | ((buf[3] as u16) << 8);

        Ok((Self {
            reader: io::BufReader::new(reader),
            last_frame: None,
            frame_size: width as u128 * height as u128,
        }, (width, height)))
    }
}

impl<R: std::io::Read> Iterator for Decoder<R> {
    type Item = Box<[u8]>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
        // Some(frame)
    }
}
