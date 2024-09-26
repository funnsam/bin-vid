fn main() {
    let file = std::fs::File::open(std::env::args().nth(1).unwrap()).unwrap();
    let (player, (width, height)) = bin_vid::Decoder::new(file).unwrap();

    println!("{width}x{height}");

    for f in player {
        let mut i = 0;
        for _ in 0..height {
            for _ in 0..width {
                print!("{}", if f[i] { "██" } else { "  " });
                i += 1;
            }

            println!();
        }
        print!("\x1b[{height}A");
        std::thread::sleep_ms(100);
    }
}
