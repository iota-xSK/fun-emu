use std::{
    array, env,
    fs::File,
    io::{self, Read},
};

use macroquad::prelude::*;

#[macroquad::main("BasicShapes")]
async fn main() -> io::Result<()> {
    let mut processor = MyProcessor::new();
    let mut bus = TextMode::new();

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("error: wrong usage \n usage: fun-emu <rom>.rom");
        return Ok(());
    }

    // Open the file
    let mut file = match File::open(&args[1]) {
        Ok(file) => file,
        Err(err) => {
            println!("Error opening file: {}", err);
            return Err(err);
        }
    };

    let rom = std::fs::read(&args[1])?;
    // file.read(&mut rom).expect("error: couldn't read file");

    println!("{:x?}", rom);

    for i in 0..rom.len() {
        bus.write(i as u16, rom[i])
    }

    loop {
        if is_key_pressed(KeyCode::Space) {
            println!("instruction: {:#x}", bus.read(processor.pc));
            println!("pc: {}", processor.pc);
            println!("reg: {:?}", processor.r);
            println!("sp: {}", processor.sp);
            println!("{}", processor.lit_mode);
            processor.step(&mut bus);
        }
        clear_background(BLACK);
        bus.render();

        next_frame().await
    }
}
struct MyProcessor {
    r: [u8; 16],
    // r0 = acc
    // r1 = x // high byte of memmory in the normal addressing mode
    // r2 = y // low byte of memmory in the normal addressing mode
    // r3 = s // status flags
    // flags
    // xxx.xuog - Underflow Overflow Greater than; x - proprietary
    // others are general purpose
    pc: u16, // program pointer
    lit_mode: bool,
    lit_register: u8,
    sp: u16,
}
impl MyProcessor {
    fn new() -> Self {
        Self {
            r: [0; 16],
            pc: 0x0100,
            sp: 1024,
            lit_mode: false,
            lit_register: 0,
        }
    }
}

trait Processor {
    fn step(&mut self, memmory: &mut impl Bus);
    fn interrupt(&mut self, addr: u16);
}

trait Bus {
    fn write(&mut self, addres: u16, data: u8);
    fn read(&mut self, addres: u16) -> u8;
}

impl Processor for MyProcessor {
    fn step(&mut self, memmory: &mut impl Bus) {
        let read = memmory.read(self.pc);
        let addr = self.r[2] as u16 | ((self.r[1] as u16) << 8);
        let low_nibble = (read & 0xf) as usize;
        if !self.lit_mode {
            match read & 0xf0 {
                0x00 => {
                    self.lit_register = low_nibble as u8;
                    self.lit_mode = true;
                }
                0x10 => match read {
                    0x10 => self.pc = addr.wrapping_sub(1),
                    0x11 => {
                        self.sp = self.sp.wrapping_sub(2);
                        memmory.write(self.sp, (self.pc & 0x00ff) as u8);
                        memmory.write(self.sp + 1, ((self.pc & 0xff00) >> 8) as u8);
                        // memmory.write(self.sp, (self.pc & 0x00ff) as u8);
                        // memmory.write(self.sp + 1, ((self.pc & 0xff00) >> 8) as u8);
                        self.pc = addr.wrapping_sub(1);
                    }
                    0x12 => {
                        self.pc = memmory.read(self.sp) as u16
                            | ((memmory.read(self.sp + 1) as u16) << 8) as u16;
                        self.sp = self.sp.wrapping_add(2);
                    }
                    0x13 => {
                        self.pc -= 1;
                    }
                    _ => (),
                },
                0x20 => {
                    if !(self.r[low_nibble] == 0) {
                        self.pc = addr.wrapping_sub(1);
                    }
                }
                0x30 => self.r[0] = self.r[(read & 0xf) as usize],
                0x40 => self.r[low_nibble] = self.r[0],
                0x50 => self.r[low_nibble] = memmory.read(addr),
                0x60 => memmory.write(addr, self.r[(read & 0xf) as usize]),
                0x70 => self.r[0] = (self.r[0] == self.r[low_nibble]) as u8,
                0x80 => {
                    self.r[0] = if self.r[3] & 0x1 == 1 {
                        if self.r[0] > self.r[low_nibble] {
                            0xff
                        } else {
                            0x00
                        }
                    } else {
                        if self.r[0] < self.r[low_nibble] {
                            0xff
                        } else {
                            0x00
                        }
                    }
                }
                0x90 => {
                    let original_value = self.r[0];
                    self.r[0] = original_value.wrapping_add(self.r[low_nibble]);
                    if let None = original_value.checked_add(self.r[low_nibble]) {
                        self.r[3] |= 1 << 1;
                    } else {
                        self.r[3] &= !(1 << 1);
                    }
                }
                0xa0 => {
                    let original_value = self.r[0];
                    self.r[0] = original_value.wrapping_sub(self.r[low_nibble]);
                    if let None = original_value.checked_sub(self.r[low_nibble]) {
                        self.r[3] |= 1 << 2;
                    } else {
                        self.r[3] &= !(1 << 2);
                    }
                }
                0xb0 => {
                    self.r[0] <<= self.r[low_nibble];
                }
                0xc0 => {
                    self.r[0] >>= self.r[low_nibble];
                }
                0xd0 => {
                    self.r[0] |= self.r[low_nibble];
                }
                0xe0 => {
                    self.r[0] &= self.r[low_nibble];
                }
                0xf0 => self.r[0] = !self.r[low_nibble],
                _ => unreachable!(),
            }
        } else {
            self.r[self.lit_register as usize] = read;
            self.lit_mode = false;
        }
        self.pc = self.pc.wrapping_add(1)
    }

    fn interrupt(&mut self, addr: u16) {
        self.pc = addr;
    }
}

struct ButtonLedEmu {
    button: bool,
    led: u8,
    memmory: [u8; 65536],
}

impl ButtonLedEmu {
    fn new() -> Self {
        Self {
            button: false,
            led: 0,
            memmory: [0; 65536],
        }
    }
}
impl Bus for ButtonLedEmu {
    fn write(&mut self, address: u16, data: u8) {
        match address {
            0 => self.led = data,
            _ => self.memmory[address as usize] = data,
        }
    }

    fn read(&mut self, address: u16) -> u8 {
        match address {
            1 => {
                if self.button {
                    1
                } else {
                    0
                }
            }
            _ => self.memmory[address as usize],
        }
    }
}

struct TextMode {
    memory: [u8; 65536],
    vram: [[u8; 80]; 25],
    row: u8,
}

impl TextMode {
    fn new() -> Self {
        Self {
            memory: array::from_fn(|_| Default::default()),
            vram: array::from_fn(|i| array::from_fn(|j| 28 + i as u8 + j as u8)),
            row: 0,
        }
    }
    fn render(&self) {
        for (i, line) in self.vram.iter().enumerate() {
            draw_text(
                &line.map(|a| a as char).iter().collect::<String>(),
                0.0,
                i as f32 * 30.0 + 30.0,
                30.0,
                WHITE,
            )
        }
    }
}

impl Bus for TextMode {
    fn write(&mut self, address: u16, data: u8) {
        match address {
            0 => {
                self.row = data;
                if self.row < 80 {
                    self.vram[self.row as usize].copy_from_slice(&self.memory[6400..6480]);
                }
                println!("here");
            }
            _ => {
                self.memory[address as usize] = data;
            }
        }
    }

    fn read(&mut self, address: u16) -> u8 {
        match address {
            3 => get_last_key_pressed().unwrap_or(KeyCode::F24) as u8,
            _ => self.memory[address as usize],
        }
    }
}

struct VectorAndController {
    commands_read: [u8; 128],
    commands_write: [u8; 128],
    memory: [u8; 65536],
    stack: [u8; 64],
    sp: i8,
}

impl Bus for VectorAndController {
    fn write(&mut self, address: u16, data: u8) {
        match address {
            0..=127 => self.commands_write[address as usize] = data,
            128 => {
                std::mem::swap(&mut self.commands_write, &mut self.commands_read);
            }
            _ => self.memory[address as usize] = data,
        }
    }

    fn read(&mut self, address: u16) -> u8 {
        match address {
            0..=127 => self.commands_write[address as usize],
            129 => {
                is_key_down(KeyCode::Up) as u8
                    | (is_key_down(KeyCode::Down) as u8) << 1
                    | (is_key_down(KeyCode::Left) as u8) << 2
                    | (is_key_down(KeyCode::Right) as u8) << 3
                    | (is_key_down(KeyCode::W) as u8) << 4
                    | (is_key_down(KeyCode::A) as u8) << 6
                    | (is_key_down(KeyCode::S) as u8) << 6
                    | (is_key_down(KeyCode::D) as u8) << 7
            }
            _ => self.memory[address as usize],
        }
    }
}

impl VectorAndController {
    fn render(&self) {
        clear_background(BLACK);
        let mut pc: u8 = 0;
        let mut pen_down = false;
        let current_positon = Vec2::new(0.0, 0.0);

        while pc < 128 {
            match pc & 0xf0 {
                0 => return,
                1 => {
                    // pos
                }
                _ => todo!(),
            }
        }
    }

    fn new() -> Self {
        Self {
            commands_read: [0; 128],
            commands_write: [0; 128],
            memory: [0; 65536],
            stack: [0; 64],
            sp: -1,
        }
    }
}
