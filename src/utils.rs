use std::{io, rc::Rc};

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

        if self.last_length >= 8 {
            self.bits.push(0);
            self.last_length = 0;
        }
    }

    pub fn write_bits(&mut self, l: usize, u: usize) {
        for i in 0..l {
            self.write_bit(u & (1 << (l - i - 1)) != 0);
        }

        /*if l == 0 { return; }

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
        }*/
    }

    pub fn write_rle_packet(&mut self, u: usize) {
        let x = u + 1;
        let msb = (usize::MAX - (usize::MAX >> 1)) >> x.leading_zeros();
        let msb_length = (usize::BITS - msb.leading_zeros()) as usize;

        self.write_bits(msb_length - 1, !1);
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

        bw.dump();
        assert_eq!(bw.bits, &[0x34, 0x12, 0x56, 0b01111_101, 0b000_10110]);
        assert_eq!(bw.last_length, 5);
    }
}

pub struct BitStream<R: io::Read> {
    pub reader: R,
    pub byte: Result<u8, Rc<io::Error>>,
    pub at: u8,
}

impl<R: io::Read> BitStream<R> {
    pub fn new(mut reader: R) -> Self {
        let mut buf = [0];
        let err = reader.read_exact(&mut buf);

        Self {
            reader,
            byte: err.map_or_else(|e| Err(e.into()), |_| Ok(buf[0])),
            at: 0,
        }
    }

    pub fn read_bit(&mut self) -> Result<bool, Rc<io::Error>> {
        let r = (self.byte.clone()? >> self.at) & 1 != 0;
        self.at += 1;

        if self.at >= 8 {
            let mut buf = [0];
            let err = self.reader.read_exact(&mut buf);
            self.byte = err.map_or_else(|e| Err(e.into()), |_| Ok(buf[0])).into();
            self.at = 0;
        }

        Ok(r)
    }


    pub fn read_rle_packet(&mut self) -> Result<usize, Rc<io::Error>> {
        let mut length = 1; // include 0 bit
        while self.read_bit()? { length += 1; }

        let mut b = 0;
        for _ in 0..length {
            b <<= 1;
            b |= self.read_bit()? as usize;
        }

        Ok(((1_usize << length) - 2) + b + 1)
    }
}
