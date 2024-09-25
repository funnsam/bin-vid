pub struct BitsWriter {
    pub bits: Vec<u8>,
    pub last_length: u8,
}

impl BitsWriter {
    pub fn new() -> Self {
        Self { bits: vec![0], last_length: 0 }
    }

    pub fn write_bit(&mut self, b: bool) {
        *self.bits.last_mut().unwrap() |= (b as u8) << self.last_length;
        self.last_length += 1;
        self.grow();
    }

    pub fn write_bits(&mut self, mut l: usize, u: u128) {
        if l == 0 { return; }

        if self.last_length != 0 {
            for _ in 0..8 - self.last_length {
                *self.bits.last_mut().unwrap() |= ((u >> (l - 1)) as u8 & 1) << self.last_length;
                self.last_length += 1;
                l -= 1;

                if l == 0 { return; }
            }

            self.last_length = 0;
            self.bits.push(0);
        }

        while l > 8 {
            self.write_u8_aligned((u >> (l - 1)) as u8);
            l -= 8;
        }

        while l != 0 {
            self.write_bit((u >> (l - 1)) & 1 != 0);
            l -= 1;
        }
    }

    pub fn write_rle_packet(&mut self, u: u128) {
        let x = u + 1;
        let msb = 0x8000_0000_0000_0000__0000_0000_0000_0000_u128 >> x.leading_zeros();
        let msb_length = (128 - msb.leading_zeros()) as usize;

        self.write_bits(msb_length - 1, 0xffff_ffff_ffff_ffff__ffff_ffff_ffff_fffe_u128);
        self.write_bits(msb_length - 1, x - msb);
    }

    pub fn write_u8_aligned(&mut self, u: u8) {
        assert_eq!(self.last_length, 0);
        *self.bits.last_mut().unwrap() = u;
        self.bits.push(0);
    }

    pub fn write_u16_aligned(&mut self, u: u16) {
        assert_eq!(self.last_length, 0);
        *self.bits.last_mut().unwrap() = u as u8;
        self.bits.push((u >> 8) as u8);
        self.bits.push(0);
    }

    pub fn grow(&mut self) {
        if self.last_length >= 8 {
            self.bits.push(0);
            self.last_length = 0;
        }
    }

    pub fn dump(&self) {
        for b in self.bits.iter() {
            print!("{b:02x} ");
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn bits_writer() {
        let mut bw = super::BitsWriter::new();

        bw.write_u16_aligned(0x1234);
        bw.write_u8_aligned(0x56);
        bw.write_bits(3, 0b101);
        bw.write_rle_packet(44);

        assert_eq!(bw.bits, &[0x34, 0x12, 0x56, 0x7d, 0x16]);
        assert_eq!(bw.last_length, 5);
    }
}

pub struct BitStream<R: std::io::Read> {
    pub reader: std::io::BufReader<R>,
    pub byte: u8,
    pub last_length: u8,
}

