//! Simple log file writer with timestamps

use crate::utils::Utils;
use crate::{Error, Result};
use chrono::{Local, Utc};
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Clone)]
pub struct LogFile {
    inner: Arc<LogFileInner>,
}

struct LogFileInner {
    state: Mutex<LogState>,
    start: Instant,
    start_seconds: f32,
    output_to_console: bool,
}

struct LogState {
    writer: BufWriter<File>,
    last_seconds: f32,
}

impl LogFile {
    pub fn new(path: Option<&str>, output_to_console: bool) -> Result<Self> {
        let path = match path {
            Some(path) if !path.is_empty() => PathBuf::from(path),
            _ => {
                let base = Utils::documents_folder()?;
                let name = Utils::date_time_filename("PicoGK_", ".log");
                base.join(name)
            }
        };

        let file = File::create(&path).map_err(|e| {
            Error::FileSave(format!("Unable to create file {}: {}", path.display(), e))
        })?;
        let writer = BufWriter::new(file);

        let inner = LogFileInner {
            state: Mutex::new(LogState {
                writer,
                last_seconds: 0.0,
            }),
            start: Instant::now(),
            start_seconds: 0.0,
            output_to_console,
        };

        let log = Self {
            inner: Arc::new(inner),
        };

        log.log(format!("Opened {}", path.display()))?;
        log.log("\n----------------------------------------\n")?;
        log.log_time()?;
        log.log("\n----------------------------------------\n")?;
        log.log("System Info:\n")?;
        log.log(format!(
            "Machine Name:         {}",
            env::var("HOSTNAME")
                .or_else(|_| env::var("COMPUTERNAME"))
                .unwrap_or_default()
        ))?;
        log.log(format!("Operating System      {}", env::consts::OS))?;
        log.log(format!("Architecture:         {}", env::consts::ARCH))?;
        log.log(format!(
            "Processor Count:      {}",
            std::thread::available_parallelism()
                .map(|c| c.get())
                .unwrap_or(0)
        ))?;
        log.log(format!(
            "Command Line:         {}",
            env::args().collect::<Vec<_>>().join(" ")
        ))?;
        log.log("\n----------------------------------------\n")?;

        Ok(log)
    }

    pub fn log(&self, message: impl AsRef<str>) -> Result<()> {
        self.inner.log_lines(message.as_ref())
    }

    pub fn log_time(&self) -> Result<()> {
        let utc = Utc::now();
        let local = Local::now();
        self.log(format!(
            "Current time (UTC): {}",
            utc.format("%Y-%m-%d %H:%M:%S (UTC)")
        ))?;
        self.log(format!(
            "Current local time: {}",
            local.format("%Y-%m-%d %H:%M:%S (%z)")
        ))?;
        Ok(())
    }
}

impl LogFileInner {
    fn log_lines(&self, message: &str) -> Result<()> {
        let seconds = self.start.elapsed().as_secs_f32() - self.start_seconds;

        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let diff = seconds - state.last_seconds;
        let prefix = format!("{:7.0}s {:6.1}+ ", seconds, diff);

        for line in message.split('\n') {
            if self.output_to_console {
                println!("{}{}", prefix, line);
            }
            state.writer.write_all(prefix.as_bytes())?;
            state.writer.write_all(line.as_bytes())?;
            state.writer.write_all(b"\n")?;
            state.writer.flush()?;
            state.last_seconds = seconds;
        }

        Ok(())
    }
}

impl Drop for LogFileInner {
    fn drop(&mut self) {
        let _ = self.log_lines("\n----------------------------------------\n");
        let _ = self.log_lines("Closing log file.");
        let _ = self.log_lines(&format!(
            "Current time (UTC): {}",
            Utc::now().format("%Y-%m-%d %H:%M:%S (UTC)")
        ));
        let _ = self.log_lines("Done.");
    }
}
