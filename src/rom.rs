pub enum RomReadError {
    TooShort,
    InvalidHeader { index: usize },
}

pub struct ROM {
    // https://www.nesdev.org/wiki/INES
    header: [u8; 16],
    trainer: Option<[u8; 512]>,
    prg_rom: Vec<u8>, // 16384 * x bytes
    chr_rom: Vec<u8>, // 8192 * y bytes
    inst_rom: Option<[u8; 8192]>,
    prom: Option<[u8; 16]>, // often missing
}

impl ROM {
    pub fn new(filebytes: Vec<u8>) -> Result<ROM, RomReadError> {
        if filebytes.len() < 16 {
            return Err(RomReadError::TooShort);
        }
        let header = &filebytes[0..16];

        // ensure header bytes are valid
        let valid_first_three = [0x4e, 0x45, 0x53, 0x1a];
        for index in 0..4 {
            if header[index] != valid_first_three[index] {
                return Err(RomReadError::InvalidHeader { index });
            }
        }

        let prg_rom_size = header[4];
        let prg_rom = Vec::with_capacity(prg_rom_size as usize);

        let chr_rom_size = header[5];
        let chr_rom = Vec::with_capacity(chr_rom_size as usize);

        // TODO: implement flags and unused padding
        todo!()
    }
}
