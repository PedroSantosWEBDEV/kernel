// src/libcore/sys/log.rs
//
// Logger for the LibertyOS kernel.

/*
	IMPORTS
*/

use alloc::collections::VecDeque;
use log::{Level, Metadata, Record};
use core::sync::atomic::{AtomicBool, Ordering};
pub use log::{debug, error, info, set_max_level, warn};
use spin::Mutex;

use crate::println;

pub static LOG: Mutex<Option<Logger>> = Mutex::new(None);

// Logger struct
pub struct Logger {
    data: VecDeque<u8>,
    size: usize,
}

// SysLogger struct
struct SysLog {
    logfunc: Mutex<fn(&Record)>,
    init: AtomicBool,
}

// Implementation of the Logger struct
impl Logger {
    // New
    pub fn new(size: usize) -> Logger {
        Logger {
            data: VecDeque::with_capacity(size),
            size,
        }
    }

    // Read
    pub fn read(&self) -> (&[u8], &[u8]) {
        self.data.as_slices()
    }

    // Write
    pub fn write(&mut self, buffer: &[u8]) {
        for &b in buffer {
            while self.data.len() + 1 >= self.size {
                self.data.pop_front();
            }

            self.data.push_back(b);
        }
    }
}

// Implementation of the log::Log trait for the SysLog struct
impl log::Log for SysLog {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let logfunc = *self.logfunc.lock();
            logfunc(record);
        }
    }

    fn flush(&self) {}
}

// Initialization
pub fn init() {
    *LOG.lock() = Some(Logger::new(1024 * 1024));
}

// Initialize logger
pub fn initlog(func: fn(&Record)) {
    if !LOGGER.init.load(Ordering::SeqCst) {
        *LOGGER.logfunc.lock() = func;

        match log::set_logger(&LOGGER) {
            Ok(_) => log::set_max_level(log::LevelFilter::Info),
            Err(e) => println!("[ERR] Could not set logger: {}", e),
        }

        LOGGER.init.store(true, Ordering::SeqCst);
    } else {
        log::info!("[INFO] Logger already initialized");
    }
}

// LOGGER
static LOGGER: SysLog = SysLog {
    logfunc: Mutex::new(|_| {}),
    init: AtomicBool::new(false),
};
