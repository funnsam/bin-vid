fn main() {
    let file = std::fs::File::open(std::env::args().nth(1).unwrap()).unwrap();
}
