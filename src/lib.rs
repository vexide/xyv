#![no_std]

use alloc::{
    borrow::ToOwned,
    boxed::Box,
    string::{String, ToString},
};
use core::{any::type_name, cell::LazyCell, fmt::Write as _, mem::take, time::Duration};

use futures_util::pending;
use hashbrown::HashMap;
use log::{Level, Metadata, Record};
use serde::Serialize;
use serde_json::Value;
use vexide::{
    core::{
        io::{stdout, Stdin, Stdout, StdoutLock, Write},
        sync::{LazyLock, Mutex, OnceLock},
        time::Instant,
    },
    prelude::*,
};

extern crate alloc;

static LOG_BUFFER: Mutex<String> = Mutex::new(String::new());
static DATA_UPDATES: LazyLock<Mutex<HashMap<String, Value>>> = LazyLock::new(Mutex::default);

#[derive(Debug, Serialize)]
struct Message {
    data: HashMap<String, Value>,
    now_sec: f64,
}

struct XYVLogger;

impl log::Log for XYVLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() >= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut buffer = LOG_BUFFER
                .try_lock()
                .expect("the log buffer should be unlocked");
            writeln!(buffer, "{} - {}", record.level(), record.args()).unwrap();
        }
    }

    fn flush(&self) {}
}

pub fn record_output<T: Serialize>(key: impl Into<String>, data: T) {
    if let Err(err) = try_record_output(key, data) {
        log::error!("Failed to log a {}: {}", type_name::<T>(), err);
    }
}

pub fn try_record_output(
    key: impl Into<String>,
    data: impl Serialize,
) -> Result<(), serde_json::Error> {
    let value = serde_json::to_value(data)?;
    let mut updates = DATA_UPDATES
        .try_lock()
        .expect("the data updates map should be unlocked");
    updates.insert(key.into(), value);
    Ok(())
}

fn flush(stdout: &mut StdoutLock, init_instant: Instant, last_flush_instant: &mut Option<Instant>) {
    let mut updates: vexide::core::sync::MutexGuard<'_, HashMap<String, Value>> = DATA_UPDATES
        .try_lock()
        .expect("the data updates map should be unlocked");
    let mut logs = LOG_BUFFER
        .try_lock()
        .expect("the log buffer should be unlocked");
    let now = Instant::now();

    // Skipping flushing if there's nothing to report helps keep serial traffic lower.

    let has_new_data = !logs.is_empty() || updates.keys().len() > 0;
    let needs_heartbeat = if let Some(last_flush) = last_flush_instant {
        now.duration_since(*last_flush) > Duration::from_millis(400)
    } else {
        true
    };

    if !has_new_data && !needs_heartbeat {
        return;
    }

    *last_flush_instant = Some(now);

    // There's no need to waste space on an empty console string.
    if !logs.is_empty() {
        updates.insert("/Console".to_string(), Value::String(logs.to_owned()));
        logs.clear();
    }

    let msg = Message {
        data: updates.clone(),
        now_sec: now.duration_since(init_instant).as_secs_f64(),
    };
    updates.clear();

    let msg_data = serde_json::to_vec(&msg).expect("the xyv message should be serializable");

    stdout.write_all(&msg_data).unwrap();
    stdout.write_all(b"\n").unwrap();
}

pub async fn init_logger() {
    let init_instant = Instant::now();
    let mut last_flush_instant = None;
    let mut stdout = stdout().lock();

    log::set_logger(Box::leak(Box::new(XYVLogger))).expect("log implementation should not be set");

    spawn(async move {
        loop {
            flush(&mut stdout, init_instant, &mut last_flush_instant);
            sleep(Duration::from_millis(20)).await;
        }
    })
    .detach();
}
