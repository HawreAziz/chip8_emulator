use core::{num, panic};
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
            (0x2, _, _, _) => {
                if self.stack_ptr as usize >= self.stack.len() {
                    panic!("Stack overflow, stack out of space");
                }
                self.stack[self.stack_ptr as usize] = self.pc;
                self.stack_ptr += 1;
                self.pc = self.inst.nnn;
            }
            (0, 0, 0xE, 0xE) => {
                debug!(
                    "{info} return from subroutine pc ({:x}) = stack_ptr {:x} = {:02x}",
                    self.pc,
                    self.stack_ptr,
                    (self.stack_ptr - 1)
                );
                if self.stack_ptr == 0 {
                    panic!("Stack overflow, stack reached 0");
                }
                self.stack_ptr -= 1;
                self.pc = self.stack[self.stack_ptr as usize];
            }
            (0x3, _, _, _) => {
                debug!(
                    "{info} Skip next instruction if V{:x} ({:02x}) == NN ({:02x})",
                    self.inst.x, self.v_reg[self.inst.x as usize], self.inst.nn
                );
                if self.v_reg[self.inst.x as usize] == self.inst.nn {
                    self.pc += 2;
                }
            }
            (0x4, _, _, _) => {
                debug!(
                    "{info} Skip next instruction if V{:x} ({:02x}) != NN ({:02x})",
                    self.inst.x, self.v_reg[self.inst.x as usize], self.inst.nn
                );
                if self.v_reg[self.inst.x as usize] != self.inst.nn {
                    self.pc += 2;
                }
            }
            (0x5, _, _, 0) => {
                debug!(
                    "{info} Skip next instruction if V{:x} ({:02x}) ==  V{:x} ({:02x})",
                    self.inst.x,
                    self.v_reg[self.inst.x as usize],
                    self.inst.y,
                    self.v_reg[self.inst.y as usize]
                );
                if self.v_reg[self.inst.x as usize] == self.v_reg[self.inst.y as usize] {
                    self.pc += 2;
                }
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
                    (self.v_reg[self.inst.x as usize].wrapping_add(self.inst.nn))
                );
                self.v_reg[self.inst.x as usize] =
                    self.v_reg[self.inst.x as usize].wrapping_add(self.inst.nn);
            }
            (0x8, _, _, 0) => {
                debug!(
                    "{info} Set V0x{:x} (0x{:02x}) to V0x{:x} (0x{:x}), result = 0x{:x}",
                    self.inst.x,
                    self.v_reg[self.inst.x as usize],
                    self.inst.y,
                    self.v_reg[self.inst.y as usize],
                    self.v_reg[self.inst.y as usize]
                );
                self.v_reg[self.inst.x as usize] = self.v_reg[self.inst.y as usize];
            }
            (0x8, _, _, 1) => {
                debug!(
                    "{info} Set V0x{:x} (0x{:02x}) to V0x{:x} |= V0x{:x} (0x{:x}), result = 0x{:x}",
                    self.inst.x,
                    self.v_reg[self.inst.x as usize],
                    self.inst.x,
                    self.inst.y,
                    self.v_reg[self.inst.y as usize],
                    self.v_reg[self.inst.x as usize] | self.v_reg[self.inst.y as usize]
                );
                self.v_reg[self.inst.x as usize] |= self.v_reg[self.inst.y as usize];
            }
            (0x8, _, _, 2) => {
                debug!(
                    "{info} Set V0x{:x} (0x{:02x}) to V0x{:x} &= V0x{:x} (0x{:x}), result = 0x{:x}",
                    self.inst.x,
                    self.v_reg[self.inst.x as usize],
                    self.inst.x,
                    self.inst.y,
                    self.v_reg[self.inst.y as usize],
                    self.v_reg[self.inst.x as usize] & self.v_reg[self.inst.y as usize]
                );
                self.v_reg[self.inst.x as usize] &= self.v_reg[self.inst.y as usize];
            }
            (0x8, _, _, 3) => {
                debug!(
                    "{info} Set V0x{:x} (0x{:02x}) to V0x{:x} ^= V0x{:x} (0x{:x}), result = 0x{:x}",
                    self.inst.x,
                    self.v_reg[self.inst.x as usize],
                    self.inst.x,
                    self.inst.y,
                    self.v_reg[self.inst.y as usize],
                    self.v_reg[self.inst.x as usize] ^ self.v_reg[self.inst.y as usize]
                );
                self.v_reg[self.inst.x as usize] ^= self.v_reg[self.inst.y as usize];
            }
            (0x8, _, _, 4) => {
                let (value, overflow) = self.v_reg[self.inst.x as usize]
                    .overflowing_add(self.v_reg[self.inst.y as usize]);
                let rest: u8 = if overflow { 1 } else { 0 };

                debug!(
                    "{info} Add V0x{:02x} (0x{:02x}) + V0x{:x} (0x{:02x}) = 0x{:02x}, set VF to {:}",
                    self.inst.x,
                    self.v_reg[self.inst.x as usize],
                    self.inst.y,
                    self.v_reg[self.inst.y as usize],
                    value,
                    rest
                );
                self.v_reg[self.inst.x as usize] = value;
                self.v_reg[0xF] = rest;
            }
            (0x8, _, _, 5) => {
                let (value, underflow) = self.v_reg[self.inst.x as usize]
                    .overflowing_sub(self.v_reg[self.inst.y as usize]);
                let rest = if underflow { 0 } else { 1 };
                debug!(
                    "{info} Add V0x{:02x} (0x{:02x}) - V0x{:x} (0x{:02x}) = 0x{:02x}, set VF to {:}",
                    self.inst.x,
                    self.v_reg[self.inst.x as usize],
                    self.inst.y,
                    self.v_reg[self.inst.y as usize],
                    value,
                    rest
                );
                self.v_reg[self.inst.x as usize] = value;
                self.v_reg[0xF] = rest;
            }
            (0x8, _, _, 6) => {
                debug!(
                    "{info} V0x{:x} (0x{:x}) >>= 1",
                    self.inst.x, self.v_reg[self.inst.x as usize]
                );
                let least_bit = self.v_reg[self.inst.x as usize] & 1;
                self.v_reg[self.inst.x as usize] >>= 1;
                self.v_reg[0xF] = least_bit;
            }
            (0x8, _, _, 7) => {
                let (value, underflow) = self.v_reg[self.inst.y as usize]
                    .overflowing_sub(self.v_reg[self.inst.x as usize]);
                let rest = if underflow { 0 } else { 1 };
                debug!(
                    "{info} V0x{:x} (0x{:x}) = V0x{:02x} (0x{:02x}) - V0x{:x} (0x{:02x}) = 0x{:02x}, set VF to {:}",
                    self.inst.y,
                    self.v_reg[self.inst.y as usize],
                    self.inst.y,
                    self.v_reg[self.inst.y as usize],
                    self.inst.x,
                    self.v_reg[self.inst.x as usize],
                    value,
                    rest
                );
                self.v_reg[self.inst.x as usize] = value;
                self.v_reg[0xF] = rest;
            }
            (0x8, _, _, 0xE) => {
                debug!(
                    "{info} V0x{:x} (0x{:x}) <<= 1",
                    self.inst.x, self.v_reg[self.inst.x as usize]
                );

                let most_sig = (self.v_reg[self.inst.x as usize] >> 7) & 1;
                self.v_reg[self.inst.x as usize] <<= 1;
                self.v_reg[0xF] = most_sig;
            }
            (0x9, _, _, 0) => {
                debug!(
                    "{info} Skip next instruction if V{:x} ({:02x}) !=  V{:x} ({:02x})",
                    self.inst.x,
                    self.v_reg[self.inst.x as usize],
                    self.inst.y,
                    self.v_reg[self.inst.y as usize]
                );
                if self.v_reg[self.inst.x as usize] != self.v_reg[self.inst.y as usize] {
                    self.pc += 2;
                }
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

                self.v_reg[0xF] = 0;
                for row in 0..self.inst.n as usize {
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
            }
            (0xF, _, 0x0, 0x7) => {
                debug!(
                    "{info} V0x{:x} (0x{:x}) = delay_timer {}",
                    self.inst.x, self.v_reg[self.inst.x as usize], self.delay_timer
                );
                self.v_reg[self.inst.x as usize] = self.delay_timer;
            }
            (0xF, _, 0x0, 0xA) => {
                debug!(
                    "{info} if Any key pressed block and wait for key input, set VX to value of key"
                );
                let mut pressed = false;
                for i in 00..self.keypad.len() {
                    if self.keypad[i] {
                        pressed = true;
                        self.v_reg[self.inst.x as usize] = i as u8;
                        break;
                    }
                }
                if !pressed {
                    self.pc -= 2;
                }
            }
            (0xF, _, 0x1, 0x5) => {
                debug!(
                    "{info} Set delay_timer {} to V{:x} (0x{:x})",
                    self.delay_timer, self.inst.x, self.v_reg[self.inst.x as usize]
                );
                self.delay_timer = self.v_reg[self.inst.x as usize];
            }
            (0xF, _, 0x1, 0x8) => {
                debug!(
                    "{info} Set sound_timer {} to V{:x} (0x{:x})",
                    self.sound_timer, self.inst.x, self.v_reg[self.inst.x as usize]
                );
                self.sound_timer = self.v_reg[self.inst.x as usize];
            }
            (0xF, _, 0x1, 0xE) => {
                debug!(
                    "{info} I (0x{:x}) += V0x{:x} (0x{:x}) = {:x}",
                    self.i,
                    self.inst.x,
                    self.v_reg[self.inst.x as usize],
                    self.i.wrapping_add(self.v_reg[self.inst.x as usize] as u16)
                );
                self.i = self.i.wrapping_add(self.v_reg[self.inst.x as usize] as u16);
            }

            (0xF, _, 0x2, 0x9) => {
                debug!(
                    "{info} Set I 0x{:x} to sprite location memory for character in V0x{:x} (0x{:x}) = 0x{:04x}",
                    self.i,
                    self.inst.x,
                    self.v_reg[self.inst.x as usize],
                    (self.v_reg[self.inst.x as usize] * 5)
                );
                // each sprite is 5 rows height
                self.i = self.v_reg[self.inst.x as usize] as u16 * 5;
            }
            (0xF, _, 0x3, 0x3) => {
                debug!("{info}");
            }
            (0xF, _, 0x5, 0x5) => {
                debug!("{info}");
            }
            (0xF, _, 0x6, 0x5) => {
                debug!("{info}");
            }
            _ => debug!("Unimplemented opcode 0x{:04x}", opcode),
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
