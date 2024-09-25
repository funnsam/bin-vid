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
    type Item = Vec<bool>;

    fn next(&mut self) -> Option<Self::Item> {
        let frame = self.0.read_next_frame().ok()??;
        Some(frame.buffer.chunks(4).map(|c| {
            c[0] as usize + c[1] as usize + c[2] as usize > 255 * 3 / 2
        }).collect())
    }
}
