//! This is the minimal firmware for an esp32c3 acting simply as a reciver for the esp-now protocol and decoding the data via postcard.
#![no_std]
#![no_main]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use esp_backtrace as _;
use esp_println::println;
use esp_wifi::esp_now::{PeerInfo, BROADCAST_ADDRESS};
use esp_wifi::{current_millis, initialize};
use hal::clock::CpuClock;
use hal::Rng;
use hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, timer::TimerGroup, Rtc};

use hal::systimer::SystemTimer;


#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

/// The struct for our sensor data. Has to be the same as on the sender side and can take a variable amount of sensors.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct SensorData {
    controller: [char; 32],
    sensors: Vec<Data>,
}

/// Single sensor data struct.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Data {
    r#type: String,
    value: f32,
}
/// This initializes the heap to be used by the allocator.
/// DANGER: If something doesn't work for no apparent reason, try decreasing the heap size if you're not using it all.
fn init_heap() {
    const HEAP_SIZE: usize = 4 * 1024;

    extern "C" {
        static mut _heap_start: u32;
    }
    unsafe {
        let heap_start = &_heap_start as *const _ as usize;
        ALLOCATOR.init(heap_start as *mut u8, HEAP_SIZE);
    }
}
#[entry]
fn main() -> ! {
    println!("Starting gateway...");
    init_heap();

    let peripherals = Peripherals::take();

    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();

    let mut rtc = {
        let mut rtc = Rtc::new(peripherals.RTC_CNTL);
        rtc.swd.disable();
        rtc.rwdt.disable();
        rtc
    };
    let timer = SystemTimer::new(peripherals.SYSTIMER).alarm0;
    initialize(
        timer,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();

    println!("Enabling wifi...");

    let (wifi, _) = peripherals.RADIO.split();
    let mut esp_now = esp_wifi::esp_now::EspNow::new(wifi).unwrap();

    println!("esp-now version {}", esp_now.get_version().unwrap());

    let mut next_send_time = current_millis() + 5 * 1000;
    loop {
        let r = esp_now.receive();
        if let Some(r) = r {
            println!("Received: {:x?}", r);
            let decoded_data: SensorData = postcard::from_bytes(&r.data).unwrap();
            if let Ok(json_serialized) = serde_json::to_string(&decoded_data) {
                println!("Decoded: {}", json_serialized);
            }
            if r.info.dst_address == BROADCAST_ADDRESS {
                if !esp_now.peer_exists(&r.info.src_address).unwrap() {
                    esp_now
                        .add_peer(PeerInfo {
                            peer_address: r.info.src_address,
                            lmk: None,
                            channel: None,
                            encrypt: false,
                        })
                        .unwrap();
                }
            }
        }
        // Since our data is deallocated here we do not have to worry about the heap growing.
    }
}
