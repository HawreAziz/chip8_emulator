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
}

impl Chip8 {
    pub fn new() -> Chip8 {
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
        };
        chip8.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        chip8
    }
}
