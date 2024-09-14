// this is an example to send a pwm wave out on a pin

//#![deny(unsafe_code)]
#![no_main]
#![no_std]

#[allow(unused_variables)]
#[allow(unused_imports)]

// Halt on panic
use panic_halt as _;

use cortex_m_rt::entry;
use stm32f4xx_hal::dma::DmaFlag;
use stm32f4xx_hal::{
    dma::{config::DmaConfig, MemoryToPeripheral, Stream0, StreamsTuple, Transfer}, 
    gpio::{gpioa, gpiob, gpioc, Speed, Alternate, Pin}, 
    pac::{self, DMA1, TIM2, interrupt}, 
    prelude::*, 
    time::Hertz, 
    timer::{Channel, Event, Timer, Channel1, Channel2},
    spi::*,
};

#[allow(clippy::empty_loop)]
#[entry]
fn main() -> ! {
    if let (Some(pac_peripherals), Some(cortex_peripherals)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        // Set up GPIOB
        let gpioa = pac_peripherals.GPIOA.split();
        
        // Configure PB15 as alternate function for TIM2 (AF1)
        let step_pin = gpioa
            .pa2
            .into_alternate::<1>() // Use AF1 for TIM2 channel
            .speed(Speed::VeryHigh);
        
        // Set up the system clock. We want to run at 84MHz.
        let rcc = pac_peripherals.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();
        let frequency = 48.MHz(); // max seems to be 20kHz for 1/16 microstepping (or 25kHz if using motor 28v at 1/16 microstepping) or 4kHz for 1/4 microstepping
        let channel = Channel1::new(gpioa.pa0);

        // Set up TIM2
        let mut timer = Timer::new(pac_peripherals.TIM2, &clocks).pwm_hz(
            channel,
            frequency,  // Set desired square wave frequency, 1 kHz here
        );

        // Enable PWM output on Channel 1 (corresponding to step_pin)
        timer.enable(Channel::C1);
        
        // Set the duty cycle to 50% (square wave)
        timer.set_duty(Channel::C1, timer.get_max_duty() / 2);
        
        loop {
            // Main loop can handle other tasks if needed
            //timer.listen(event)
        }
    }

    loop {}
    
}

static mut PULSE_COUNTER: u32 = 0;

// Interrupt handler for TIM2
#[interrupt]
fn TIM2() {
    unsafe {
        // Increment the pulse counter
        PULSE_COUNTER += 1;

        // Check if 3200 pulses have been reached
        if PULSE_COUNTER >= 3200 {
            // Disable TIM2
            pac::Peripherals::steal().TIM2.cr1.modify(|_, w| w.cen().clear_bit());
        }

        // Clear the interrupt flag
        pac::Peripherals::steal().TIM2.sr.modify(|_, w| w.uif().clear_bit());
    }
}
