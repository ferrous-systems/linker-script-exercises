# `linker-script-exercises`

## Required tools

- `nm`, `size`

## Exercise 1: baseline

``` console
$ cargo build

$ size -A -x target/thumbv7em-none-eabihf/debug/app
section              size         addr
.vector_table       0x100          0x0
.text               0x564        0x100
.rodata             0x180        0x664
.data                 0x0   0x20000000
.bss                  0x0   0x20000000
.uninit               0x0   0x20000000
```

Things to check

- Compare this output to `memory.x`
  - `memory.x` is provided by the HAL and placed in `target` directory
  - e.g. `target/thumbv7em-none-eabihf/debug/build/nrf52840-hal-cf89422e8bc97512/out/memory.x`
  - check the `MEMORY` command in the linker script

Questions to answer:
- what are the memory regions specified in the `memory.x` linker script? 
- which sections from the `size` output goes into which memory region?

## Exercise 2: inspect `link.x`

- goal: why does each section goes into RAM or FLASH?
- each section has to be assigned to a memory region
  - that is done in `link.x`
  - memory regions are defined in `memory.x`
  - `link.x` is provided by `cortex-m-rt`

- Questions to answer
  - the locations of linker sections, do they make sense? do they go into the 'linker section to memory region' mapping specified in `link.x`?

## Exercise 3: order of linker sections

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

## create static variables

``` rust
// constants go in .rodata  
static INTEGER_IN_RODATA: u32 = 0;
static STRING_IN_RODATA: &str = "hello";
```

``` rust
// goes in .bss  if initial value is zero
static IN_BSS: u32 = 0;

// goes in .data  if initial value is NOT zero
static IN_DATA: u32 = 1;
```

## Exercise 4: override `memory.x`

- if you change `RAM` to `RAM2` -> linker error

- instructions
  - copy `memory.x` into the the root of the repository (outside `target` directory)
    - linker search for linker script in current directory and *then* in linker search path

## Exercise N: `#[link_section]` attribute

- TODO show use of `.uninit`

``` rust
// by default goes into .bss
static BUFFER: [u8; 1024] = [0; 1024];

// will go into .uninit section; will NOT be initialized at startup
#[link_section = ".uninit.BUFFER"]
static BUFFER: MaybeUninit<[u8; 1024]> = MaybeUninit::uninit();
```

- use case: initial chunk of memory for allocator (`#[global_allocator]`) or memory pool (see `heapless::Pool`)
  - why? avoid initializing that chunk of memory on startup = faster start up times
