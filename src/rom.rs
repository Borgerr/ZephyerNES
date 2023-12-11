pub struct ROM {
    // https://www.nesdev.org/wiki/INES
    header: [u8; 16],
    trainer: Option([u8; 512]),
    prg_rom: Vec<u8>, // 16384 * x bytes
    chr_rom: Vec<u8>, // 8192 * y bytes
    inst_rom: Option([u8; 8192]),
    prom: Option([u8; 16]), // often missing
}

impl ROM {
    pub fn new(filebytes: Vec<u8>) -> this {}
}
