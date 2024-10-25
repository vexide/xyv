#![no_std]
#![no_main]

use core::time::Duration;

use vexide::{core::time::Instant, prelude::*};

#[vexide::main]
async fn main(p: Peripherals) {
    xyv::init_logger().await;
    let start_time = Instant::now();
    let controller = p.primary_controller;

    loop {
        let state = controller.state().unwrap_or_default();

        let elapsed = start_time.elapsed().as_secs_f64();
        let volts = elapsed.sin();
        xyv::record_output("/Arm/OutputVolts", volts);

        if state.button_a.is_now_pressed() {
            log::info!("button a pressed");
        }

        sleep(Controller::UPDATE_INTERVAL).await;
    }
}
