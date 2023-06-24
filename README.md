# V1X ISA
V1X is a small and simple fantasy cpu architecture.
It can address 65536 bytes of memory. Each instruction
is encoded in just a single byte (except for the 
literal instruction). It has 16 registers labled r0-rf.
r0 is the accumulator register. All mathematical and 
logical operations results are stored there. r1 is the
low byte of the memory address. r2 is the high byte of 
the memory address. They are used for the load and store
instruction. r3 is a flag register. More about the flags 
is written in the following section. The cpu also of course
has a 16 bit program pointer called pp for short.

## registers
### r0
accumulator register.
instructions that write to it:
 - add
 - sub
 - lsf
 - rsf
 - or
 - and
 - not
 - eq
 - tac

### r1
high byte of memory address
### r2
low byte of memory address
### r3
flag register:
- flags: xxx.xuog - 
*u*nderflow; *o*verflow; *g*reater than; x - reserved for 
future use
### r4-rf
general purpose

## instructions
### format
The high nybble of each byte is the opcode for the instruction.
The low nybble is the argument. An importatnt exception to this
is the literal instruction which takes in a second byte as its
second argument (the literal value itself).

#### 0 - lit r l
reg[r] = l
sets the register r to the value l.
It takes 2 cpu cylces to fully finish this instruction.
#### 1 - jmp _
jumps to address stored in r1 and r2, concatenating them with r1 as 
the high byte nd r2 as the low byte.
the second argument, _, should be kept 0 to ensure compatability with 
the yet unreleased and not fully designed V2X.

#### 2 - cjmp r
pp = r2 | (r1 << 8) if register r is nonzero
jumps to address stored in r1 and r2, concatenating them with r1 as the 
high byte and r2 as the low byte if value in the register r is nonzero

#### 3 - tac r
r0 = reg[r]
copies the value of register r into the accumulator register r0 (to accumulator)

#### 4 - tre r
reg[r] = reg[r0]
copies the value of register r0 into the register r (to register)

#### 5 - r r
reg[r] = mem[r2 | (r1 << 8)]
reads value stored in memory on address stored in r1 and r2 into register r.

#### 6 - w r
mem[r2 | (r1 << 8)] = reg[r] 
writes value stored in register to the memory.

#### 7 - eq r
if r0 and r have the equal value stored in them, it writes 0xff into r0, else it writes 0x00.

#### 8 - cmp r
if magic bit g in r4 is set:
  if reg[r] > reg[r0] writes 0xff into r0, else it writes 0x00
else 
  if reg[r] < reg[r0] writes 0xff into r0, else it writes 0x00

#### 9 - add r
r0 = reg[r] + reg[r0]
Sets overflow bit if it overflows. Unsets it if it doesn't.

#### a - sub
r0 = reg[r0] - reg[r] 
Sets underflow bit if it underflows. Unsets it if it doesn't.

#### b - lsf r
r0 = reg[r0] << reg[r] 

#### c - rsf r
r0 = reg[r0] >> reg[r] 

#### d - or r
bitwise or between r0 and reg[r] written into r0

#### e - and r
bitwise and between r0 and reg[r] written into r0

#### f - not r
bitwise not of reg[r] written into r0

