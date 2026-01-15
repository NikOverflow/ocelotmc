use std::io::{self, Read, Write};

pub struct PacketBuffer<'a> {
    cursor: &'a [u8],
}
impl<'a> PacketBuffer<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { cursor: data }
    }
}
impl<'a> Read for PacketBuffer<'a> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.cursor.read(buffer)
    }
}

pub struct PacketWriter {
    data: Vec<u8>,
}
impl PacketWriter {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    pub fn build(self) -> Vec<u8> {
        self.data
    }
}
impl Write for PacketWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.data.write(buffer)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.data.flush()
    }
}
