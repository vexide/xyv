#![no_std]
#![no_main]

use core::time::Duration;

use vexide::prelude::*;

#[vexide::main]
async fn main(p: Peripherals) {
    xyv::init_logger().await;
    let controller = p.primary_controller;

    loop {
        let controller = controller.left_stick.y().unwrap_or_default();
        let output_volts = controller * Motor::MAX_VOLTAGE;
        xyv::record_output("/Arm/OutputVolts", output_volts);
        log::info!("setting arm to {} volts", output_volts);

        sleep(Controller::UPDATE_INTERVAL).await;
    }
}
