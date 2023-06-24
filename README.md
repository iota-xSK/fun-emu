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
has a 16 bit program pointer called pc for short. A 16 bit
stack pointer is used to keep track of the stack.

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
**u**nderflow; **o**verflow; **g**reater than; **x** - reserved for 
future use
### r4-rf
general purpose

## instructions
### format
The high nybble of each byte is the opcode for the instruction.
The low nybble is the argument. An importatnt exception to this
is the literal instruction which takes in a second byte as its
second argument (the literal value itself). A different exception
is the instructions that have 0x1x as their opcode. They take in
no arguments.

### 0_ - lit r l
reg[r] = l
sets the register r to the value l.
It takes 2 cpu cylces to fully finish this instruction.
### 10 - jmp
jumps to address stored in r1 and r2, concatenating them with r1 as 
the high byte nd r2 as the low byte.

### 11 - call
jumps to address stored in r1 and r2, concatenating them with r1 as 
the high byte nd r2 as the low byte. Pushes the pc + 1 onto the stack
in little endian. Increments stack pointer by 1.

### 12 - ret
jumps to addres stored on the top of the stack, decrements stack pointer by 1.

### 12 - ret
pop the top 2 values off the stack and jump to that address (in little endian)

### 13 - halt
decrements pc which effectively stops execution.

### 1b (reserved)
reserved for future use
### 1c (reserved)
reserved for future use
### 1d (reserved)
reserved for future use
### 1e (reserved)
reserved for future use
### 1f (reserved)
reserved for future use


### 2_ - cjmp r
pc = r2 | (r1 << 8) if register r is nonzero
jumps to address stored in r1 and r2, concatenating them with r1 as the 
high byte and r2 as the low byte if value in the register r is nonzero

### 3_ - tac r
r0 = reg[r]
copies the value of register r into the accumulator register r0 (to accumulator)

### 4_ - tre r
reg[r] = reg[r0]
copies the value of register r0 into the register r (to register)

### 5_ - r r
reg[r] = mem[r2 | (r1 << 8)]
reads value stored in memory on address stored in r1 and r2 into register r.

### 6_ - w r
mem[r2 | (r1 << 8)] = reg[r] 
writes value stored in register to the memory.

### 7_ - eq r
if r0 and r have the equal value stored in them, it writes 0xff into r0, else it writes 0x00.

### 8_ - cmp r
if magic bit g in r4 is set:
  if reg[r] > reg[r0] writes 0xff into r0, else it writes 0x00
else 
  if reg[r] < reg[r0] writes 0xff into r0, else it writes 0x00

### 9_ - add r
r0 = reg[r] + reg[r0]
Sets overflow bit if it overflows. Unsets it if it doesn't.

### a_ - sub
r0 = reg[r0] - reg[r] 
Sets underflow bit if it underflows. Unsets it if it doesn't.

### b_ - lsf r
r0 = reg[r0] << reg[r] 

### c_ - rsf r
r0 = reg[r0] >> reg[r] 

### d_ - or r
bitwise or between r0 and reg[r] written into r0

### e_ - and r
bitwise and between r0 and reg[r] written into r0

### f_ - not r
bitwise not of reg[r] written into r0

