// this is an example to send a pwm wave out on a pin

//#![deny(unsafe_code)]
#![no_main]
#![no_std]

#[allow(unused_variables)]
#[allow(unused_imports)]

// Halt on panic
use panic_halt as _;

use cortex_m_rt::entry;
use cortex_m::interrupt::Mutex;
use core::cell::RefCell;
use stm32f4xx_hal::dma::DmaFlag;
use stm32f4xx_hal::{
    dma::{config::DmaConfig, MemoryToPeripheral, Stream0, StreamsTuple, Transfer}, 
    gpio::{self, gpioa, gpiob, gpioc, Speed, Alternate, Pin, PushPull, Output}, 
    pac::{self, DMA1, TIM2, interrupt, Interrupt}, 
    prelude::*, 
    time::Hertz, 
    timer::{Channel, Event, Timer, Channel1, Channel2, CounterUs},
    spi::*,
};

#[allow(clippy::empty_loop)]
#[entry]
fn main() -> ! {
    if let (Some(pac_peripherals), Some(cortex_peripherals)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        // set up clocks
        let rcc = pac_peripherals.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.MHz()).pclk1(48.MHz()).freeze();
        
        // Set up GPIOA
        let gpioa = pac_peripherals.GPIOA.split();
        
        // Configure PA5 pin to blink LED
        let mut step = gpioa.pa5.into_push_pull_output().speed(Speed::VeryHigh);
        step.set_high();

        // Move the pin into our global storage
        cortex_m::interrupt::free(|cs| *G_STEP.borrow(cs).borrow_mut() = Some(step));

        // Set up a timer expiring after 1s
        let mut timer = pac_peripherals.TIM2.counter(&clocks);
        let freq  = 1.micros();

        timer.start(freq).unwrap();

        // Generate an interrupt when the timer expires
        timer.listen(Event::Update);

        // Move the timer into our global storage
        cortex_m::interrupt::free(|cs| *G_TIM.borrow(cs).borrow_mut() = Some(timer));

        //enable TIM2 interrupt
        unsafe {
            cortex_m::peripheral::NVIC::unmask(Interrupt::TIM2);
        }
        
        loop {
            // Main loop can handle other tasks if needed
            //timer.listen(event)
        }
    }

    loop {}
    
}


// A type definition for the GPIO pin to be used for the step pulses
type StepPin = gpio::PA5<Output<PushPull>>;

// Make STEP pin globally available
static G_STEP: Mutex<RefCell<Option<StepPin>>> = Mutex::new(RefCell::new(None));

// Make timer interrupt registers globally available
static G_TIM: Mutex<RefCell<Option<CounterUs<TIM2>>>> = Mutex::new(RefCell::new(None));


// Define an interrupt handler, i.e. function to call when interrupt occurs.
// This specific interrupt will "trip" when the timer TIM2 times out
// this seems to cap out at a frequency of 25KHz. perhaps the mutex overhead is too much?
#[interrupt]
fn TIM2() {
    unsafe {
        static mut STEP: Option<StepPin> = None;
        static mut TIM: Option<CounterUs<TIM2>> = None;

        let step = STEP.get_or_insert_with(|| {
            cortex_m::interrupt::free(|cs| {
                // Move LED pin here, leaving a None in its place
                G_STEP.borrow(cs).replace(None).unwrap()
            })
        });

        let tim = TIM.get_or_insert_with(|| {
            cortex_m::interrupt::free(|cs| {
                // Move LED pin here, leaving a None in its place
                G_TIM.borrow(cs).replace(None).unwrap()
            })
        });

        step.toggle();
        let _ = tim.wait();

        // Clear the interrupt flag
        //pac::Peripherals::steal().TIM2.sr.modify(|_, w| w.uif().clear_bit());
    }
}
