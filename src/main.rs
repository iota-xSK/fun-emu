use crossterm::event::poll;
use crossterm::event::read;
use crossterm::terminal::*;
use crossterm::QueueableCommand;
use crossterm::{cursor, execute, ExecutableCommand};
use std::thread;
use std::time::Duration;
use std::{
    array, env,
    io::{self, stdout, Write},
};

fn main() -> io::Result<()> {
    let mut processor = MyProcessor::new();
    let mut bus = TextMode::new();

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("error: wrong usage \n usage: fun-emu <rom>.rom");
        return Ok(());
    }

    let rom = std::fs::read(&args[1])?;

    for i in 0..rom.len() {
        bus.write(i as u16, rom[i])
    }

    execute!(stdout(), EnterAlternateScreen)?;

    loop {
        // println!("instruction: {:#x}", bus.read(processor.pc));
        // println!("pc: {}", processor.pc);
        // println!("reg: {:?}", processor.r);
        // println!("sp: {}", processor.sp);
        // println!("{}", processor.lit_mode);
        processor.step(&mut bus);
        thread::sleep(Duration::from_millis(1));
        bus.render()?;
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
    fn render(&self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Bus for TextMode {
    fn write(&mut self, address: u16, data: u8) {
        match address {
            0 => {
                self.row = data;
                if self.row < 80 {
                    self.vram[self.row as usize].copy_from_slice(&self.memory[6400..6480]);
                    execute!(stdout(), crossterm::cursor::MoveTo(0, self.row as u16))
                        .expect("couldn't write to stdout");
                    write!(
                        stdout(),
                        "{}",
                        self.vram[self.row as usize]
                            .iter()
                            .map(|a| *a as char)
                            .collect::<String>()
                    )
                    .expect("couldn't write");
                }
            }
            _ => {
                self.memory[address as usize] = data;
            }
        }
    }

    fn read(&mut self, address: u16) -> u8 {
        match address {
            2 => {
                if poll(Duration::from_secs(0)).unwrap() {
                    match read() {
                        Ok(event) => match event {
                            crossterm::event::Event::FocusGained => todo!(),
                            crossterm::event::Event::FocusLost => todo!(),
                            crossterm::event::Event::Key(_) => todo!(),
                            _ => self.memory[2],
                        },
                        Err(err) => {
                            println!("{}", err);
                            panic!()
                        }
                    }
                } else {
                    self.memory[2]
                }
            }
            _ => self.memory[address as usize],
        }
    }
}
