use chrono::{DateTime, Local};

pub fn error(error_type: &str, message: &str, cause: &str) {
  println!(
    "\x1b[2m({d})\x1b[0m [\x1b[31m{error_type}\x1b[0m]: \x1b[31m{message}\n                         Cause: {cause}\x1b[0m",
    d = format_date(Local::now(), "%d/%m/%Y %H:%M:%S")
  )
}

pub fn error_without_cause(error_type: &str, message: &str) {
  println!(
    "\x1b[2m({d})\x1b[0m [\x1b[31m{error_type}\x1b[0m]: \x1b[31m{message}\x1b[0m",
    d = format_date(Local::now(), "%d/%m/%Y %H:%M:%S")
  )
}

pub fn error_help(error_type: &str, message: &str, help: &str) {
  println!(
    "\x1b[2m({d})\x1b[0m [\x1b[31m{error_type}\x1b[0m]: \x1b[31m{message}\n                         Help: {help}\x1b[0m",
    d = format_date(Local::now(), "%d/%m/%Y %H:%M:%S")
  )
}

pub fn warn(warn_type: &str, message: &str) {
  println!(
    "\x1b[2m({d})\x1b[0m [\x1b[33m{warn_type}\x1b[0m]: \x1b[33m{message}\x1b[0m",
    d = format_date(Local::now(), "%d/%m/%Y %H:%M:%S"),
  )
}

pub fn warn_with_cause(warn_type: &str, message: &str, cause: &str) {
  println!(
    "\x1b[2m({d})\x1b[0m [\x1b[33m{warn_type}\x1b[0m]: \x1b[33m{message}\n                         Cause: {cause}\x1b[0m",
    d = format_date(Local::now(), "%d/%m/%Y %H:%M:%S")
  )
}

pub fn info(info_type: &str, message: &str) {
  println!(
    "\x1b[2m({d})\x1b[0m [\x1b[34m{info_type}\x1b[0m]: \x1b[34m{message}\x1b[0m",
    d = format_date(Local::now(), "%d/%m/%Y %H:%M:%S")
  )
}

pub fn info_with_detail(info_type: &str, message: &str, details: &str) {
  println!(
    "\x1b[2m({d})\x1b[0m [\x1b[34m{info_type}\x1b[0m]: \x1b[34m{message}\n                         Details: {details}\x1b[0m",
    d = format_date(Local::now(), "%d/%m/%Y %H:%M:%S")
  )
}

pub fn success(success_type: &str, message: &str) {
  println!(
    "\x1b[2m({d})\x1b[0m [\x1b[32m{success_type}\x1b[0m]: \x1b[32m{message}\x1b[0m",
    d = format_date(Local::now(), "%d/%m/%Y %H:%M:%S"),
  )
}

pub fn send(msg_type: &str, message: &str, type_color: i32) {
  println!(
    "\x1b[2m({d})\x1b[0m [\x1b[{type_color}m{msg_type}\x1b[0m]: {message}\x1b[0m",
    d = format_date(Local::now(), "%d/%m/%Y %H:%M:%S")
  )
}

pub fn security(info_type: &str, message: &str) {
  println!(
    "\x1b[2m({d})\x1b[0m [\x1b[36m{info_type}\x1b[0m]: \x1b[36m{message}\x1b[0m",
    d = format_date(Local::now(), "%d/%m/%Y %H:%M:%S")
  )
}

pub fn security_with_detail(info_type: &str, message: &str, details: &str) {
  println!(
    "\x1b[2m({d})\x1b[0m [\x1b[36m{info_type}\x1b[0m]: \x1b[36m{message}\n                         Details: {details}\x1b[0m",
    d = format_date(Local::now(), "%d/%m/%Y %H:%M:%S")
  )
}

/// "%d/%m/%Y %H:%M:%S"
pub fn format_date(date: DateTime<Local>, format: &str) -> String {
  format!("{}", date.format(format))
}

// ======================================================================================================
use thiserror::Error;

/// Possible failure cases for [get_id()].
#[derive(Debug, Error)]
pub enum HwIdError {
    /// Could not detect a hardware id. This might be caused
    /// by a misconfigured system or by this feature not
    /// being supported by the system or platform.
    #[error("no HWID was found on system")]
    NotFound,
    /// Found a putative HWID, but something was wrong with
    /// it.  The `String` argument contains a path or other
    /// identifier at which the HWID was found. This will
    /// usually indicate something is really wrong with the
    /// system.
    #[error("{0:?}: contains malformed HWID")]
    Malformed(String),
}

#[cfg(target_os = "windows")]
pub mod hwid {
    use super::*;
    use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_QUERY_VALUE};

    /// Get the hardware ID of this machine. The HWID is
    /// obtained from the Windows registry at location
    /// `\\\\SOFTWARE\\Microsoft\\Cryptography\\MachineGuid`.
    pub fn get_id() -> Result<std::string::String, HwIdError> {
        // escaping is fun, right? right???
        let hive = winreg::RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey_with_flags("Software\\Microsoft\\Cryptography", KEY_QUERY_VALUE)
            .or(Err(HwIdError::NotFound))?;
        let id = hive.get_value("MachineGuid").or(Err(HwIdError::NotFound))?;
        Ok(id)
    }
}

#[cfg(target_os = "linux")]
pub mod hwid {
    use super::*;

    /// Get the hardware ID of this machine. The HWID is
    /// obtained from `/var/lib/dbus/machine-id`, or failing
    /// that from `/etc/machine-id`.
    pub fn get_id() -> Result<std::string::String, HwIdError> {
        let paths = ["/var/lib/dbus/machine-id", "/etc/machine-id"];
        for p in paths {
            if let Ok(id_contents) = std::fs::read_to_string(p) {
                let id_str = id_contents
                    .lines()
                    .next()
                    .ok_or_else(|| HwIdError::Malformed(id_contents.to_string()))?;
                return Ok(id_str.to_string());
            }
        }
        Err(HwIdError::NotFound)
    }
}

#[cfg(target_os = "freebsd")]
#[cfg(target_os = "dragonfly")]
#[cfg(target_os = "openbsd")]
#[cfg(target_os = "netbsd")]
pub mod hwid {
    pub fn get_id() -> std::string::String {
        unimplemented!("*BSD support is not implemented")
    }
}