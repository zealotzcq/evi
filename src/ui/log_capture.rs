use parking_lot::Mutex;
use std::sync::Arc;

static LOG_BUFFER: parking_lot::Mutex<Option<Arc<Mutex<Vec<String>>>>> =
    parking_lot::Mutex::new(None);

pub fn init_log_capture(buffer: Arc<Mutex<Vec<String>>>) {
    *LOG_BUFFER.lock() = Some(buffer);
}

pub struct CaptureLogger;

impl log::Log for CaptureLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let msg = format!("{} [{}] {}", ts_now(), record.level(), record.args());
            eprintln!("{}", msg);
            if let Some(buf) = LOG_BUFFER.lock().as_ref() {
                let mut v = buf.lock();
                v.push(msg.clone());
                if v.len() > 5000 {
                    v.drain(..1000);
                }
            }
            write_log_file(&msg);
        }
    }

    fn flush(&self) {}
}

fn log_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| String::new());
    if home.is_empty() {
        std::path::PathBuf::from("/tmp/.evi.log")
    } else {
        std::path::Path::new(&home).join(".evi.log")
    }
}

fn write_log_file(msg: &str) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path())
    {
        let _ = writeln!(f, "{}", msg);
        let _ = f.flush();
    }
}

fn ts_now() -> String {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = d.as_secs();
    let ms = d.subsec_millis();
    let tod = secs % 86400;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        tod / 3600,
        (tod % 3600) / 60,
        tod % 60,
        ms
    )
}
