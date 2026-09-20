use core::num;
use std::{fs::File, io::Read};

use log::debug;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

const SCALE: u32 = 15;
pub const WINDOW_WIDTH: u32 = SCREEN_WIDTH as u32 * SCALE;
pub const WINDOW_HEIGHT: u32 = SCREEN_HEIGHT as u32 * SCALE;

const RAM_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const REG_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;
const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(Debug)]
struct Inst {
    x: u8,
    y: u8,
    n: u8,
    nn: u8,
    nnn: u16,
}

impl Inst {
    fn new() -> Inst {
        Inst {
            x: 0,
            y: 0,
            n: 0,
            nn: 0,
            nnn: 0,
        }
    }
}

#[derive(Debug)]
pub struct Chip8 {
    ram: [u8; RAM_SIZE],
    display: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    pc: u16,
    i: u16,
    stack: [u16; STACK_SIZE],
    stack_ptr: u16,
    delay_timer: u8,
    sound_timer: u8,
    v_reg: [u8; REG_SIZE],
    keypad: [bool; NUM_KEYS],
    inst: Inst,
}

impl Chip8 {
    pub fn new(chip8_file: &str) -> Chip8 {
        env_logger::init();
        let mut chip8 = Chip8 {
            ram: [0; RAM_SIZE],
            display: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            pc: START_ADDR,
            i: 0,
            stack: [0; STACK_SIZE],
            stack_ptr: 0,
            delay_timer: 0,
            sound_timer: 0,
            v_reg: [0; REG_SIZE],
            keypad: [false; NUM_KEYS],
            inst: Inst::new(),
        };
        chip8.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        chip8.load_chip(chip8_file);
        chip8
    }

    fn load_chip(&mut self, chip8_file: &str) {
        let mut file = File::open(chip8_file).expect("Failed to open file {chip8_file}");
        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer)
            .expect("Could not load {chip8_file} content");

        let start = START_ADDR as usize;
        let end = START_ADDR as usize + buffer.len();
        self.ram[start..end].copy_from_slice(buffer.as_slice());
    }

    pub fn get_display(&self) -> &[bool] {
        &self.display
    }

    fn extract_opcode(&mut self) -> u16 {
        let high_byte = self.ram[self.pc as usize] as u16;
        let low_byte = self.ram[self.pc as usize + 1] as u16;
        self.pc += 2;
        (high_byte << 8) | low_byte
    }

    pub fn execute(&mut self) {
        let opcode = self.extract_opcode();

        // 0xFFFF
        //   cxyn
        let c = ((opcode >> 12) & 0xF) as u8;
        self.inst.x = ((opcode >> 8) & 0xF) as u8;
        self.inst.y = ((opcode >> 4) & 0xF) as u8;
        self.inst.n = (opcode & 0xF) as u8;
        self.inst.nn = (opcode & 0x00FF) as u8;
        self.inst.nnn = (opcode & 0x0FFF) as u16;

        let info = format!("Address 0x{:04x}, Opcode 0x{:04x} Desc: ", self.pc, opcode);
        match (c, self.inst.x, self.inst.y, self.inst.n) {
            (0, 0, 0xE, 0) => {
                debug!("{info} clear the display set all pixels off to 0");
                self.display = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }
            (0x1, _, _, _) => {
                debug!(
                    "{info} Set PC (V0x{:02x}) to NN (0x{:02x})",
                    self.pc, self.inst.nnn
                );
                self.pc = self.inst.nnn;
            }
            (0x6, _, _, _) => {
                debug!(
                    "{info} Set V0x{:02x} (0x{:02x}) to NN (0x{:02x})",
                    self.inst.x, self.v_reg[self.inst.x as usize], self.inst.nn
                );
                self.v_reg[self.inst.x as usize] = self.inst.nn;
            }
            (0x7, _, _, _) => {
                debug!(
                    "{info} Set register V0x{:02x} (0x{:02x}) += NN (0x{:02x}) = result = 0x{:02x}",
                    self.inst.x,
                    self.v_reg[self.inst.x as usize],
                    self.inst.nn,
                    (self.v_reg[self.inst.x as usize] + self.inst.nn)
                );
                self.v_reg[self.inst.x as usize] =
                    self.v_reg[self.inst.x as usize].wrapping_add(self.inst.nn);
            }
            (0xA, _, _, _) => {
                debug!(
                    "{info} Set index register I ({:04x}) to NNN ({:04x})",
                    self.i, self.inst.nnn
                );
                self.i = self.inst.nnn;
            }
            (0xD, _, _, _) => {
                debug!("{info} Render sprite at x and y");
                // extract x and cordinates
                let x_coords = self.v_reg[self.inst.x as usize] as usize % SCREEN_WIDTH;
                let y_coords = self.v_reg[self.inst.y as usize] as usize % SCREEN_HEIGHT;
                let num_rows: usize = self.inst.n as usize;

                self.v_reg[0xF] = 0;
                for row in 0..num_rows {
                    let sprite_byte = self.ram[self.i as usize + row];
                    for col in 0..8 {
                        if (sprite_byte & (0b10000000 >> col)) != 0 {
                            let pixel_x = (x_coords + col) % SCREEN_WIDTH;
                            let pixel_y = (y_coords + row) % SCREEN_HEIGHT;
                            let index = pixel_x + SCREEN_WIDTH * pixel_y;
                            if self.display[index] {
                                self.v_reg[0xF] = 1
                            }
                            self.display[index] ^= true;
                        }
                    }
                }

                // loop in N rows
                // extract each row
                // loop through all the bits
                // if dispaly at x,y is on and pixel bit is also on turn off and set VF = 1
                // make sure your handler the edge cases
            }
            _ => debug!("Unimplemented opcode {:04x}", opcode),
        }
    }
}

pub struct Config {
    pub bg_color: u32,
    pub fg_color: u32,
    pub screen_width: usize,
    pub screen_height: usize,
    pub window_height: u32,
    pub window_width: u32,
    pub scale: u32,
}

impl Config {
    pub fn new() -> Config {
        Config {
            bg_color: 0x000000FF,
            fg_color: 0xFFFFFFFF,
            screen_width: SCREEN_WIDTH,
            screen_height: SCREEN_HEIGHT,
            window_width: WINDOW_WIDTH,
            window_height: WINDOW_HEIGHT,
            scale: SCALE,
        }
    }
}
