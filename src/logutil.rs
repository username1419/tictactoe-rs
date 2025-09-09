use std::fs::OpenOptions;
use std::io::Write;

#[allow(unused)]
pub enum LogStatus {
    DEBUG,
    INFO,
    WARN,
    ERROR,
    FATAL,
}

pub fn log(status: LogStatus, message: &str) {
    let line = format!(
        "[{}][{}]: {}",
        match status {
            LogStatus::DEBUG => "DEBUG",
            LogStatus::INFO => "INFO",
            LogStatus::WARN => "WARN",
            LogStatus::ERROR => "ERROR",
            LogStatus::FATAL => "FATAL",
        },
        chrono::Local::now().format("%Y/%m/%d; %H:%M:%S"),
        message
    );

    // this is a really bad idea for optimization but i cant be fucked to write a singleton rn
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(".log");

    if file.is_err() {
        return;
    }

    writeln!(file.unwrap(), "{}", line).ok();
}
