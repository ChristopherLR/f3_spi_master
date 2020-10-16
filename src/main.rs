#![no_main]
#![no_std]
#[allow(unused_extern_crates)] // NOTE(allow) bug rust-lang/rust53964

extern crate panic_halt;
use stm32f3xx_hal as hal;
use cortex_m_rt::{entry, exception, ExceptionFrame};

use hal::{
    pac,
    prelude::*,
    spi::{ Mode, Phase, Polarity, Spi },
};

/// SETUP: STM32F3DISCOVERY -> ARDUINO MEGA
/// MOSI: PA7 -> 51
/// MISO: PA6 -> 50
/// SCK:  PA5 -> 52
/// NSS:  PA4 -> 53

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    
    // Clock Config:
    let clocks = rcc
        .cfgr
        // External oscillator to 16 Mhz
        .use_hse(16.mhz())
        // System clock to 48 Mhz
        .sysclk(48.mhz())
        // Setting APB bus to 24 Mhz
        .pclk1(24.mhz())
        .freeze(&mut flash.acr);

    // Slave Select set to a push pull output:
    // HIGH = Start Tx
    // LOW  = Stop Tx
    let mut nss = gpioa.pa4.into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
    // Configuring SPI pins
    let sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
    let mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);

    // Setting SPI to MODE_0
    // CPHA - First Clock Transition
    // CPOL - Clock Polarity
    let spi_mode = Mode {
        polarity: Polarity::IdleLow,
        phase: Phase::CaptureOnFirstTransition,
    };

    let mut spi = Spi::spi1(dp.SPI1, (sck, miso, mosi), spi_mode, 8.mhz(), clocks, &mut rcc.apb2);

    loop {
        // Create an `u8` array, which can be transfered via SPI.
        let mut msg_send = [b'H', b'E', b'L', b'L', b'O', b'\n'];
        // Start SS low then transmit
        match nss.set_low() {
            Ok(()) => {
                let msg_received = spi.transfer(&mut msg_send).unwrap();
                for _ in 0..100 {
                    continue;
                }
            },
            Err(_) => ()
        }
        // End transmission
        while nss.is_set_low().unwrap() {
            nss.set_high();
        }
    }
}


#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
