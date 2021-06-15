#![no_std]
#![no_main]

use defmt_rtt as _; // defmt transport
use nrf52840_hal as _; // interrupt handlers
use panic_probe as _;  // panic handler

static VARIABLE: u32 = 0;
// static VARIABLE: &str = "hello";

#[cortex_m_rt::entry]
fn main() -> ! {
    let address = &VARIABLE as *const _;
    defmt::info!("addrof(VARIABLE) = {}", address);

    exit()
}

fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt(); 
    }
}