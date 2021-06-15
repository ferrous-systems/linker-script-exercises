# `linker-script-exercises`

## Required tools

``` console
$ rustup component add llvm-tools-preview

$ cargo install cargo-binutils

$ cargo install flip-link
```

## `size`, linker sections and `MEMORY`

``` console
$ cargo build

$ rust-size -A -x target/thumbv7em-none-eabihf/debug/app
app  :
section                size         addr
.vector_table         0x100          0x0
.text                 0xc3c        0x100
.rodata               0x3a0        0xd3c
.data                  0x30   0x20000000
.bss                    0x4   0x20000030
.uninit               0x400   0x20000034

$ # shortcut
$ cargo size -- -A -x
```

Things to check

- Compare this output to `memory.x`
  - `memory.x` is provided by the HAL and placed in `target` directory
  - e.g. `target/thumbv7em-none-eabihf/debug/build/nrf52840-hal-cf89422e8bc97512/out/memory.x`
  - check the `MEMORY` command in the linker script

Questions to answer:
- what are the memory regions specified in the `memory.x` linker script? 
- which sections from the `size` output goes into which memory region?

## `nm`, symbols

``` console
$ cargo nm -- --demangle --numeric-sort
00000100 T Reset
(..)
00000196 T main
000001a0 t app::__cortex_m_rt_main::h2d5f5dd012e8b171
(..)
00000d3c r app::VARIABLE::hca59953dc6886887
(..)
20040000 A _stack_start
```

Questions to answer:
- in which sections are the above symbols (functions / static variables) located?

add this to `main`

``` rust
let array = [0u8; 16];
let address = array.as_ptr();
defmt::info!("addrof(array) = {}", address1);
```

- how is this address related to `_stack_start`?
- try creating more stack variables OR
  - try doing some function calls and printing the address of stack variables allocated there

## `.vector_table`

``` console
$ cargo readobj -- -x .vector_table
Hex dump of section '.vector_table':
0x00000000 00000420 01010000 2f020000 270d0000 ... ..../...'...
(..)
```

Questions to answer:
- how do the first 2 32-bit words relate to the symbols output from before?
  - NOTE due to endianness the 32-bit word is printed reversed

## inspect `link.x`

- goal: why does each section goes into RAM or FLASH?
- each section has to be assigned to a memory region
  - that is done in `link.x`
  - memory regions are defined in `memory.x`
  - `link.x` is provided by `cortex-m-rt`

``` text
SECTIONS {
  .section_name start_address {
    /* .. */
  } > MEMORY_REGION
}
```

- Questions to answer
  - the locations of linker sections, do they make sense? do they go into the 'linker section to memory region' mapping specified in `link.x`?

## location of static variables

``` rust
static VARIABLE: u32 = 0;
// static VARIABLE: &str = "hello";
```

- run `cargo nm`

``` console
$ cargo nm -- --demangle --numeric-sort 
```

- is `VARIABLE` in `FLASH` or `RAM`?
- does address reported by `nm` match the address reported by the program?
- in which linker section is the variable located?

- change `VARIABLE` to be a `&str`

- look for variable (by address) in `nm`; use address from program's output

``` console
$ cargo nm -- --demangle --numeric-sort --print-size
```

- second column is symbol size. does it match the string size?
- in which linker section is the variable located?

try these variants
``` rust
//static VARIABLE: AtomicU32 = AtomicU32::new(0);
//static VARIABLE: AtomicU32 = AtomicU32::new(1);
```

``` console
$ cargo nm -- --demangle --numeric-sort --print-size
```

- is `VARIABLE` in `FLASH` or `RAM`?
- check `size` and `nm` output. in which linker section is `VARIABLE`?

## override `memory.x`

1. change LENGTH
- copy `memory.x` from `target` directory into the root of repo
    - linker search for linker script in current directory and *then* in linker search path

- change LENGTH of `RAM` in `memory.x`
- then modify `main.rs` to force a relinking

- run `cargo nm`. look for `_stack_start`

2. change ORIGIN
- try to modify ORIGIN OF `RAM`
  - expected change: address of .bss and .data would change

3. delete RAM region
- try changing `RAM` to `RAM2`
- what happens after `cargo build`?

## use `flip-link`

- do `cargo build`
- answer these questions
  - where is the stack located?
  - where are the static variables located (.bss & .data)?

- modify `.cargo/config.toml`. add this inside `rustflags` array
``` toml
  "-C", "linker=flip-link",
```

- do `cargo build`
- answer these questions
  - where is the stack located?
  - where are the static variables located (.bss & .data)?

- TODO link to flip-link README here

## `#[link_section]` attribute

- **NOTE** `link_section` is **UNSAFE**

- try this

``` rust
// #[link_section = ".uninit.VARIABLE"]
static VARIABLE: MaybeUninit<[u8; 1024]> = MaybeUninit::uninit();

let address = VARIABLE.as_ptr();
defmt::info!("addrof(VARIABLE) = {}", address);
```

- look at `nm` and `size`. in which linker section is `VARIABLE` placed?
- uncomment `link_section` and try again

- use case: initial chunk of memory for allocator (`#[global_allocator]`) or memory pool (see `heapless::Pool`)
  - why? avoid initializing that chunk of memory on startup = faster start up times

## overriding symbols: change location of the stack

- override `memory.x`
- uncomment `_stack_start` line
- set the value to 

``` text
_stack_start = ORIGIN(RAM) + LENGTH(RAM) / 2; 
```

- run `cargo nm` and verify that `_stack_start` changed

use case
- place the stack on a different memory region
  - so that when stack overflows it doesn't overwrite static variables (`flip-link` also prevents that)

## order of linker sections

- start address of linker section in defined in `link.x` after linker section name

```
  .section_name start_address {
    /* .. */
  } > MEMORY_REGION
```

- in `link.x`, start address of `.text` is set to end address of `.vector_table` (line 83, cortex-m-rt v0.6.14)

try:
- override the `_stext` symbol to change the location of the `.text` section
- check the start address of each linker section in `link.x`

questions to answer:
- how are start addresses related to the order of the linker sections in the output of `size` 
