use std::collections::BTreeMap;
use std::fs::{self, DirEntry, File};
use std::io::{prelude::*, BufReader};
use std::os::unix::prelude::*;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serialport::{open_with_settings, posix::TTYPort, SerialPort, SerialPortSettings};

use super::errors::Result;

lazy_static! {
    pub static ref BaudRate: Vec<u32> = vec![1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200];
    pub static ref DataBits: BTreeMap<&'static str, &'static str> = {
        let mut items = BTreeMap::new();
        items.insert("Five", "5 bits per character");
        items.insert("Six", "6 bits per character");
        items.insert("Seven", "7 bits per character");
        items.insert("Eight", "8 bits per character");
        items
    };
    pub static ref FlowControl: BTreeMap<&'static str, &'static str> = {
        let mut items = BTreeMap::new();
        items.insert("None", "No flow control");
        items.insert("Software", "Flow control using XON/XOFF bytes");
        items.insert("Hardware", "Flow control using RTS/CTS signals");
        items
    };
    pub static ref Parity: BTreeMap<&'static str, &'static str> = {
        let mut items = BTreeMap::new();
        items.insert("None", "No parity bit.");
        items.insert("Odd", "Parity bit sets odd number of 1 bits");
        items.insert("Even", "Parity bit sets even number of 1 bits");
        items
    };
    pub static ref StopBits: BTreeMap<&'static str, &'static str> = {
        let mut items = BTreeMap::new();
        items.insert("One", "One stop bit");
        items.insert("Two", "Two stop bit");
        items
    };
}

pub struct Pseudo {
    master: Arc<Mutex<TTYPort>>,
    slave: TTYPort,
}

impl Pseudo {
    pub fn new() -> Result<Self> {
        let (mut master, mut slave) = TTYPort::pair()?;
        info!(
            "master tty fd: {}, path: {:?}, settings: {:?}",
            master.as_raw_fd(),
            master.name(),
            slave.settings()
        );
        info!(
            "slave  tty fd: {}, path: {:?}, settings: {:?}",
            slave.as_raw_fd(),
            slave.name(),
            slave.settings()
        );
        master.set_exclusive(false)?;
        slave.set_exclusive(false)?;
        Ok(Self {
            master: Arc::new(Mutex::new(master)),
            slave: slave,
        })
    }
    pub fn name(&self) -> Option<String> {
        self.slave.name()
    }
    pub fn setttings(&self) -> SerialPortSettings {
        self.slave.settings()
    }
    pub fn start<P: AsRef<Path>>(
        &self,
        path: P,
        interval: Duration,
        delay: Option<Duration>,
    ) -> Result<()> {
        let cb = |it: &str| -> Result<()> {
            let port = self.master.clone();
            match port.lock() {
                Ok(mut port) => {
                    if let Some(delay) = delay {
                        let mut buf: Vec<u8> = vec![0; 1 << 10];
                        let len = port.read(buf.as_mut_slice())?;
                        info!(
                            "receive {} bytes: {}",
                            len,
                            std::str::from_utf8(&buf[..len])?
                        );
                        thread::sleep(delay);
                    }
                    let len = port.write(it.as_bytes())?;
                    info!("send {} bytes: {}", len, it);
                }
                Err(e) => {
                    error!("failed in get serial port: {:?}", e);
                }
            }

            thread::sleep(interval);
            Ok(())
        };
        protocols(path.as_ref(), &cb)?;

        Ok(())
    }
}

pub fn publisher<P: AsRef<Path>>(
    path: P,
    name: &String,
    settings: &SerialPortSettings,
    interval: Duration,
    delay: Option<Duration>,
) -> Result<()> {
    let port = open_serial_port(name, settings)?;
    let port = Arc::new(Mutex::new(port));
    let cb = |it: &str| -> Result<()> {
        thread::sleep(interval);
        let port = port.clone();
        match port.lock() {
            Ok(mut port) => {
                let len = port.write(it.as_bytes())?;
                info!("send {} bytes: {}", len, it);
                if let Some(delay) = delay {
                    thread::sleep(delay);
                    let mut buf: Vec<u8> = vec![0; 1 << 10];
                    let len = port.read(buf.as_mut_slice())?;
                    info!(
                        "receive {} bytes: {}",
                        len,
                        std::str::from_utf8(&buf[..len])?
                    );
                }
            }
            Err(e) => {
                error!("failed in get serial port: {:?}", e);
            }
        }

        Ok(())
    };
    protocols(path.as_ref(), &cb)?;

    Ok(())
}

fn open_serial_port(
    name: &String,
    settings: &SerialPortSettings,
) -> Result<Box<serialport::SerialPort>> {
    info!("open {} with {:?}", name, settings);
    let port = open_with_settings(name, settings)?;
    Ok(port)
}

fn protocols<P: AsRef<Path>>(path: P, cb: &Fn(&str) -> Result<()>) -> Result<()> {
    let cb = |it: &DirEntry| -> Result<()> {
        let file = it.path();
        info!("load from file {}", file.display());
        let fd = File::open(file)?;
        let br = BufReader::new(fd);

        for line in br.lines() {
            let line = line?;
            let line = line.trim();
            if !line.is_empty() {
                cb(line)?;
            }
        }
        Ok(())
    };
    visit_dirs(&path.as_ref(), &cb)?;

    Ok(())
}

fn visit_dirs(dir: &Path, cb: &Fn(&DirEntry) -> Result<()>) -> Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry)?;
            }
        }
    }
    Ok(())
}

// #[derive(Debug, Clone)]
// pub struct Tty {
//     pub name: String,
//     pub baud_rate: u32,
//     pub timeout: u64,
// }
//
// impl Tty {
//     pub fn listen<F>(&self, delay: Duration, func: F) -> Result<()>
//     where
//         F: Fn(Vec<u8>) -> Result<()>,
//     {
//         let mut port = self.open()?;
//
//         loop {
//             thread::sleep(delay);
//
//             let mut buf: Vec<u8> = vec![0; 1 << 10];
//             let len = port.read(buf.as_mut_slice())?;
//             debug!("receive {} bytes", len);
//
//             func(buf[..len].to_vec())?;
//         }
//     }
//
//     pub fn send(&self, message: &[u8], delay: Duration) -> Result<Vec<u8>> {
//         let mut port = self.open()?;
//
//         let len = port.write(message)?;
//         debug!("send {} bytes ", len);
//         thread::sleep(delay);
//
//         let mut buf: Vec<u8> = vec![0; 1 << 10];
//         let len = port.read(buf.as_mut_slice())?;
//         debug!("receive {} bytes", len);
//         Ok(buf[..len].to_vec())
//     }
//
//     fn open(s: &SerialPortSettings) -> Result<Box<serialport::SerialPort>> {
//         let mut settings = SerialPortSettings::default();
//         settings.timeout = Duration::from_secs(self.timeout);
//         settings.baud_rate = self.baud_rate.into();
//         debug!("open {:?}", &settings);
//         let port = open_with_settings(&self.name, &settings)?;
//         Ok(port)
//     }
// }
//
