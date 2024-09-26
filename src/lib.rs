use std::io;

mod utils;

pub struct VideoInfo {
    pub width: u16,
    pub height: u16,
    pub fps: u16,
}

pub fn encode_video<'a, S: 'a + Iterator<Item = Box<[bool]>>>(info: VideoInfo, stream: S) -> Vec<u8> {
    let mut bits = utils::BitsWriter::new();
    bits.write_u16_aligned(info.width);
    bits.write_u16_aligned(info.height);
    bits.write_u16_aligned(info.fps);

    let mut last_frame: Option<Box<[bool]>> = None;

    for frame in stream {
        assert_eq!(frame.len(), info.width as usize * info.height as usize);

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
    bits.write_bit(true); // I-frame indication
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
    bits.write_bit(false); // P-frame indication
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
    reader: utils::BitStream<R>,
    last_frame: Box<[bool]>,
    frame_size: usize,
}

impl<R: io::Read> Decoder<R> {
    pub fn new(mut reader: R) -> Result<(Self, VideoInfo), io::Error> {
        let mut buf = [0; 6];
        reader.read_exact(&mut buf)?;

        let width = buf[0] as u16 | ((buf[1] as u16) << 8);
        let height = buf[2] as u16 | ((buf[3] as u16) << 8);
        let fps = buf[4] as u16 | ((buf[5] as u16) << 8);
        let frame_size = width as usize * height as usize;

        Ok((Self {
            reader: utils::BitStream::new(reader),
            last_frame: vec![false; frame_size].into(),
            frame_size,
        }, VideoInfo { width, height, fps }))
    }
}

impl<R: std::io::Read> Iterator for Decoder<R> {
    type Item = Box<[bool]>;

    fn next(&mut self) -> Option<Self::Item> {
        let is_i_frame = self.reader.read_bit().ok()?;
        let mut color = self.reader.read_bit().ok()?;

        if is_i_frame {
            self.last_frame.fill(false);
        }

        let mut size = 0;
        while self.frame_size > size {
            let l = self.reader.read_rle_packet().ok()?;
            for i in size..size + l {
                self.last_frame[i] ^= color;
            }
            size += l;
            color ^= true;
        }

        assert_eq!(self.frame_size, size);
        Some(self.last_frame.clone())
    }
}
