#![no_std]
#![no_main]

use core::time::Duration;

use vexide::{core::time::Instant, prelude::*};

#[vexide::main]
async fn main(p: Peripherals) {
    xyv::init_logger().await;
    let start_time = Instant::now();
    let mut controller = p.primary_controller;

    loop {
        let elapsed = start_time.elapsed().as_secs_f64();
        let volts = elapsed.sin();
        xyv::record_output("/Arm/OutputVolts", volts);

        let a = controller.button_a.was_pressed().unwrap_or_default();
        if a {
            log::info!("button a pressed");
        }

        sleep(Controller::UPDATE_INTERVAL).await;
    }
}
