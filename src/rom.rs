pub enum RomReadError {
    TooShort,
    InvalidHeader { index: usize },
}

pub struct ROM {
    // https://www.nesdev.org/wiki/INES
    header: [u8; 16],
    trainer: Option([u8; 512]),
    prg_rom: [u8], // 16384 * x bytes
    chr_rom: [u8], // 8192 * y bytes
    inst_rom: Option([u8; 8192]),
    prom: Option([u8; 16]), // often missing
}

impl ROM {
    pub fn new(filebytes: Vec<u8>) -> Result<this, RomReadError> {
        if filebytes.len() < 16 {
            return err(RomReadError::TooShort);
        }
        let header: [u8; 16] = filebytes[0..16];

        // ensure header bytes are valid
        let valid_first_three: [u8; 4] = [0x4e, 0x45, 0x53, 0x1a];
        for i in 0..4 {
            if header[i] != valid_first_three[i] {
                return err(RomReadError::InvalidHeader(i));
            }
        }

        let prg_rom_size = header[4];
        let prg_rom = [u8; prg_rom_size];

        let chr_rom_size = header[5];
        let chr_rom = [u8; chr_rom_size];

        // TODO: implement flags and unused padding
        todo!()
    }
}
