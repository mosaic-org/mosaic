use std::{
    fs,
    io::{self, prelude::*},
    os::unix::io::RawFd,
    path::PathBuf,
};

use crate::utils::consts::{MOSAIC_TMP_LOG_DIR, MOSAIC_TMP_LOG_FILE};

pub fn atomic_create_file(file_name: &str) {
    let _ = fs::OpenOptions::new().create(true).open(file_name);
}

pub fn atomic_create_dir(dir_name: &str) -> io::Result<()> {
    if let Err(e) = fs::create_dir(dir_name) {
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            Ok(())
        } else {
            Err(e)
        }
    } else {
        Ok(())
    }
}

pub fn debug_log_to_file(mut message: String) -> io::Result<()> {
    message.push('\n');
    debug_log_to_file_without_newline(message)
}

pub fn debug_log_to_file_without_newline(message: String) -> io::Result<()> {
    atomic_create_file(MOSAIC_TMP_LOG_FILE);
    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(MOSAIC_TMP_LOG_FILE)?;
    file.write_all(message.as_bytes())
}

pub fn _debug_log_to_file_pid_3(message: String, pid: RawFd) -> io::Result<()> {
    if pid == 3 {
        debug_log_to_file(message)
    } else {
        Ok(())
    }
}

#[allow(dead_code)]
pub fn delete_log_file() -> io::Result<()> {
    if fs::metadata(MOSAIC_TMP_LOG_FILE).is_ok() {
        fs::remove_file(MOSAIC_TMP_LOG_FILE)
    } else {
        Ok(())
    }
}

#[allow(dead_code)]
pub fn delete_log_dir() -> io::Result<()> {
    if fs::metadata(MOSAIC_TMP_LOG_DIR).is_ok() {
        fs::remove_dir_all(MOSAIC_TMP_LOG_DIR)
    } else {
        Ok(())
    }
}

pub fn debug_to_file(message: u8, pid: RawFd) -> io::Result<()> {
    let mut path = PathBuf::new();
    path.push(MOSAIC_TMP_LOG_DIR);
    path.push(format!("mosaic-{}.log", pid.to_string()));

    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)?;
    file.write_all(&[message])
}
