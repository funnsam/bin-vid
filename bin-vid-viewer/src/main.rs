fn main() {
    let file = std::fs::File::open(std::env::args().nth(1).unwrap()).unwrap();
    let (player, video) = bin_vid::Decoder::new(file).unwrap();

    for f in player {
        let mut i = 0;
        for _ in 0..video.height {
            for _ in 0..video.width {
                print!("{}", if f[i] { "██" } else { "  " });
                i += 1;
            }

            println!();
        }
        print!("\x1b[{}A", video.height);
        std::thread::sleep(std::time::Duration::from_secs_f64(1.0 / video.fps as f64));
    }
}
