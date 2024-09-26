fn main() {
    let mut decoder = gif::DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);

    let file = std::fs::File::open(std::env::args().nth(1).unwrap()).unwrap();
    let decoder = decoder.read_info(file).unwrap();

    let size = (decoder.width(), decoder.height());
    let result = bin_vid::encode_video(size, FrameStream(decoder));

    println!("Compressed to {} bytes", result.len());
    std::fs::write("video.bin", result).unwrap();
}

struct FrameStream<R: std::io::Read>(gif::Decoder<R>);

impl<R: std::io::Read> Iterator for FrameStream<R> {
    type Item = Box<[bool]>;

    fn next(&mut self) -> Option<Self::Item> {
        let frame = self.0.read_next_frame().ok()??;
        let width = frame.width;
        let height = frame.height;
        let mut buffer = frame.buffer.clone();
        let mut out: Box<[_]> = vec![false; buffer.len() / 4].into();

        for i in 0..buffer.len() / 4 {
            buffer.to_mut()[i] = ((
                buffer[4 * i + 0] as usize +
                buffer[4 * i + 1] as usize +
                buffer[4 * i + 2] as usize
            ) / 3) as u8;
        }

        for y in 0..height {
            for x in 0..width {
                let idx = |x, y| width as usize * y as usize + x as usize;
                let old = buffer[idx(x, y)] as isize;
                let new = old > 255 / 2;
                out[idx(x, y)] = new;
                let new = new as isize * 255;

                let error = old - new;

                if x + 1 < width {
                    buffer.to_mut()[idx(x + 1, y)] = buffer[idx(x + 1, y)].saturating_add((error * 7 / 16) as _);
                }
                if y + 1 < height {
                    buffer.to_mut()[idx(x, y + 1)] = buffer[idx(x, y + 1)].saturating_add((error * 5 / 16) as _);

                    if x != 0 {
                        buffer.to_mut()[idx(x - 1, y + 1)] = buffer[idx(x - 1, y + 1)].saturating_add((error * 3 / 16) as _);
                    }
                    if x + 1 < width {
                        buffer.to_mut()[idx(x + 1, y + 1)] = buffer[idx(x + 1, y + 1)].saturating_add((error * 1 / 16) as _);
                    }
                }
            }
        }

        Some(out)
    }
}
