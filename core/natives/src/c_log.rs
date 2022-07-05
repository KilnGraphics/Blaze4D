//! Forwards rust logs to some external handler.

use std::panic::catch_unwind;
use std::process::exit;
use log::{Level, LevelFilter, Log, Metadata, Record};

// target_ptr, msg_ptr, target_len, msg_len, level
type PfnLog = unsafe extern "C" fn(*const u8, *const u8, u32, u32, u32);

struct CLogger {
    pfn: PfnLog,
}

impl CLogger {
    fn new(pfn: PfnLog) -> Self {
        Self {
            pfn,
        }
    }

    fn log_internal(&self, target: &str, message: &str, level: Level) {
        let level = match level {
            Level::Error => 4,
            Level::Warn => 3,
            Level::Info => 2,
            Level::Debug => 1,
            Level::Trace => 0,
        };

        let target = target.as_bytes();
        let message = message.as_bytes();

        unsafe {
            (self.pfn)(target.as_ptr(), message.as_ptr(), target.len() as u32, message.len() as u32, level);
        }
    }
}

impl Log for CLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if let Some(msg) = record.args().as_str() {
            self.log_internal(record.target(), msg, record.level());
        } else {
            self.log_internal(record.target(), &record.args().to_string(), record.level());
        }
    }

    fn flush(&self) {
    }
}

#[no_mangle]
unsafe extern "C" fn b4d_init_external_logger(pfn: PfnLog) {
    catch_unwind(|| {
        let logger = Box::new(CLogger::new(pfn));
        log::set_boxed_logger(logger).unwrap_or_else(|err| {
            println!("Failed to set logger in b4d_init_external_logger. {:?}", err);
            exit(1);
        });

        log::set_max_level(LevelFilter::Info);
    }).unwrap_or_else(|_| {
        // Log is not going to work here so we use print instead
        println!("panic in b4d_init_external_logger");
        exit(1);
    })
}