#[derive(Debug)]
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

    mapper_number: u16,

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

        let mut prg_rom_size = header[4] as usize;
        let mut chr_rom_size = header[5] as usize;

        let mut trainer: Option<[u8; 512]> = None;

        // Flags 6
        let vertical_mirroring = (header[6] & 0b1) == 1;
        let prg_ram_present = (header[6] & 0b10) >> 1 == 1;
        if (header[6] & 0b100) >> 2 == 1 {
            // 512-byte trainer at $7000~$71FF
            if filebytes.len() >= 16 + 512 {
                trainer = Some(filebytes[16..16 + 512].try_into().unwrap());
            } else {
                return Err(RomReadError::InvalidHeader { index: 6 });
            }
        }
        let four_screen_vram = (header[6] & 0b1000) >> 3 == 1;

        // mapper number kind of between flags 6 and 7
        // if NES 2.0, this only captures D0..D7
        let mut mapper_number = (header[7] as u16 & 0xf0) | ((header[6] as u16 & 0xf0) >> 4);

        // Flags 7
        if ((header[7] & 0b1100) >> 2) == 0b10 {
            // flags 8-15 are in NES 2.0 format

            // Flags 8
            mapper_number |= ((header[8] as u16) & 0xf) << 8;
            // submapper is the upper nibble here
            // need to decide what to do with it
            match header[8] & 0xf0 >> 4 {
                _ => {}
            }

            // Flags 9
            prg_rom_size |= (header[9] as usize & 0xf) << 8;
            chr_rom_size |= (header[9] as usize & 0xf0) << 4;

            // Flags 10...
        } else {
            // flags 8-15 are in INES format

            // Flags 9 and 10 left unused by emulator
            // and rest of header bytes are irrelevant
        }

        // determine if exponent multiplier notation is used for PRG/CHR-ROM
        if chr_rom_size >> 8 == 0xf {
            let multiplier = chr_rom_size & 0b11;
            let exponent = (chr_rom_size & 0x0ff) >> 2;

            // actual CHR-ROM size is 2^E * (MM*2+1)
            chr_rom_size = (0b1 << exponent) * (multiplier * 2 + 1);
        }
        if prg_rom_size >> 8 == 0xf {
            let multiplier = prg_rom_size & 0b11;
            let exponent = (prg_rom_size & 0x0ff) >> 2;

            // actual PRG-ROM size is 2^E * (MM*2+1)
            prg_rom_size = (0b1 << exponent) * (multiplier * 2 + 1);
        }

        let chr_rom = Vec::with_capacity(chr_rom_size);
        let prg_rom = Vec::with_capacity(prg_rom_size);

        Ok(CartridgeData {
            trainer,
            prg_rom,
            chr_rom,
            mapper_number,
            vertical_mirroring,
            four_screen_vram,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::matches;

    use crate::memory::CartridgeData;

    fn valid_header_no_data(size: usize) -> Vec<u8> {
        let mut rom = vec![0; 16 + size];
        // let valid_first_four = [0x4e, 0x45, 0x53, 0x1a];
        rom[0] = 0x4e;
        rom[1] = 0x45;
        rom[2] = 0x53;
        rom[3] = 0x1a;

        rom
    }

    #[test]
    fn returns_valid_with_blank_header() {
        let rom = valid_header_no_data(0);

        assert!(matches!(CartridgeData::new(rom), Result::Ok(..)));
    }

    #[test]
    fn returns_invalid_no_data() {
        assert!(matches!(CartridgeData::new(Vec::new()), Result::Err(..)));
    }

    #[test]
    fn returns_valid_with_no_trainer() {
        let rom = valid_header_no_data(0);
        let data = CartridgeData::new(rom).unwrap();

        assert!(matches!(data.trainer, Option::None));
    }

    #[test]
    fn returns_valid_with_trainerflag_and_data() {
        let mut rom = valid_header_no_data(512);
        rom[6] = 0b100;

        let data = CartridgeData::new(rom).unwrap();

        assert!(matches!(data.trainer, Option::Some(..)));
    }

    #[test]
    fn returns_invalid_with_trainerflag_and_no_data() {
        let mut rom = valid_header_no_data(0);
        rom[6] = 0b100;

        assert!(matches!(CartridgeData::new(rom), Result::Err(..)));
    }

    #[test]
    fn arbitrary_mapper_number_ines() {
        let mapper_num = 0x3e;

        let mut rom = valid_header_no_data(0);
        rom[6] = 0xe0;
        rom[7] = 0x30; // INES header format

        let data = CartridgeData::new(rom).unwrap();

        assert_eq!(mapper_num, data.mapper_number);
    }

    #[test]
    fn arbitrary_mapper_number_nes2() {
        let mapper_num = 0xa3e;

        let mut rom = valid_header_no_data(0);
        rom[6] = 0xe0;
        rom[7] = 0x30 | 0b1000; // NES 2.0 header format
        rom[8] = 0xa;

        let data = CartridgeData::new(rom).unwrap();

        assert_eq!(mapper_num, data.mapper_number);
    }

    #[test]
    fn vertical_mirroring() {
        let mut rom = valid_header_no_data(0);
        rom[6] = 0b1;

        let data = CartridgeData::new(rom).unwrap();

        assert!(data.vertical_mirroring);
    }

    #[test]
    fn horizontal_mirroring() {
        let rom = valid_header_no_data(0);
        // rom[6] = 0;

        let data = CartridgeData::new(rom).unwrap();

        assert!(!data.vertical_mirroring);
    }

    #[test]
    fn four_screen_on() {
        let mut rom = valid_header_no_data(0);
        rom[6] = 0b1000;

        let data = CartridgeData::new(rom).unwrap();

        assert!(data.four_screen_vram);
    }

    #[test]
    fn four_screen_off() {
        let rom = valid_header_no_data(0);
        // rom[6] = 0;

        let data = CartridgeData::new(rom).unwrap();

        assert!(!data.four_screen_vram);
    }
}
