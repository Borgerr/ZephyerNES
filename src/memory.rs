pub enum RomReadError {
    TooShort,
    InvalidHeader { index: usize },
}

pub struct CartridgeData {
    // https://www.nesdev.org/wiki/INES
    // https://www.nesdev.org/wiki/NES_2.0
    //header: [u8; 16],
    trainer: Option<[u8; 512]>,
    prg_rom: Vec<u8>, // 16384 * x bytes
    chr_rom: Vec<u8>, // 8192 * y bytes
    inst_rom: Option<[u8; 8192]>,
    prom: Option<[u8; 16]>, // often missing

    vertical_mirroring: bool, // true if vertical, false if horizontal
    four_screen_vram: bool,   // if true, ignore vertical_mirroring
}

impl CartridgeData {
    pub fn new(filebytes: Vec<u8>) -> Result<CartridgeData, RomReadError> {
        if filebytes.len() < 16 {
            return Err(RomReadError::TooShort);
        }
        let header = &filebytes[0..16];

        // ensure header bytes are valid
        let valid_first_four = [0x4e, 0x45, 0x53, 0x1a];
        for index in 0..4 {
            if header[index] != valid_first_four[index] {
                return Err(RomReadError::InvalidHeader { index });
            }
        }

        let prg_rom_size = header[4];
        let prg_rom = Vec::with_capacity(prg_rom_size as usize);

        let chr_rom_size = header[5];
        let chr_rom = Vec::with_capacity(chr_rom_size as usize);

        let mut trainer = None;

        // Flags 6
        let vertical_mirroring = (header[6] & 0b1) == 1;
        if (header[6] & 0b01) >> 1 == 1 {
            // cartridge contains battery-backed PRG RAM ($6000~7FFF)
            // or other persistent memory
        }
        if (header[6] & 0b001) >> 2 == 1 {
            // 512-byte trainer at $7000~$71FF
            let trainer_arr = &filebytes[16..16 + 512].try_into();
            trainer = Some(trainer_arr);
        }
        if (header[6] & 0b0001) >> 3 == 1 {
            // Ignore mirroring control or mirroring bit;
            // instead provide four-screen VRAM
        }

        // Flags 8

        // Flags 7, 9 and 10 left unused by emulator
        // and rest of header bytes are irrelevant
    }
}
