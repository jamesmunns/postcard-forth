//! This example shows how to use UART (Universal asynchronous receiver-transmitter) in the RP2040 chip.
//!
//! No specific hardware is specified in this example. Only output on pin 0 is tested.
//! The Raspberry Pi Debug Probe (https://www.raspberrypi.com/products/debug-probe/) could be used
//! with its UART port.

#![no_std]
#![no_main]

use core::hint::black_box;

use embassy_executor::Spawner;
// use embassy_rp::uart;
use static_cell::ConstStaticCell;
use {defmt_rtt as _, panic_probe as _};
mod gen;

static IN_BUF: ConstStaticCell<[u8; 64 * 1024]> = ConstStaticCell::new([0u8; 64 * 1024]);
static OUT_BUF: ConstStaticCell<[u8; 64 * 1024]> = ConstStaticCell::new([0u8; 64 * 1024]);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_rp::init(Default::default());
    // let config = uart::Config::default();
    let in_buf = IN_BUF.take();
    let out_buf = OUT_BUF.take();

    gen::round_trip_all(
        in_buf,
        out_buf,
        |x| {
            black_box(x);
        },
        |x| {
            black_box(x);
        },
        |x| {
            black_box(x);
        },
    ).unwrap();

    // let mut uart = uart::Uart::new_with_rtscts_blocking(p.UART0, p.PIN_0, p.PIN_1, p.PIN_3, p.PIN_2, config);
    // uart.blocking_write("Hello World!\r\n".as_bytes()).unwrap();

    // loop {
    //     uart.blocking_write("hello there!\r\n".as_bytes()).unwrap();
    //     cortex_m::asm::delay(1_000_000);
    // }
}
