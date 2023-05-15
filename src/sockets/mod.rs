use anyhow::{anyhow, bail, Result};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

#[derive(Debug)]
#[repr(C)]
pub struct SktWrapper {
    name: String,
    listener: TcpListener,
    stream: Vec<TcpStream>,
}

impl SktWrapper {
    fn accept(&mut self) -> Result<usize> {
        match self.listener.accept() {
            Ok((stream, addr)) => {
                stream.set_nonblocking(true)?;
                self.stream.push(stream);
                let hello = format!(
                    "[{}-{}] Conneted to {:?}!\n",
                    self.name,
                    self.stream.len(),
                    self.listener.local_addr()?
                );
                let i = self.stream.len() - 1;
                self.stream[i].write_all(&hello.as_bytes())?;
                println!("[{}] new client: {:?}", self.name, addr);
                Ok(self.stream.len())
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    Ok(0)
                } else {
                    Err(anyhow!(e))
                }
            }
        }
    }

    pub fn write(&mut self, id: usize, data: &[u8]) -> Result<usize> {
        if id > self.stream.len() {
            bail!(format!("[{}] client {} not exist!", self.name, id));
        }
        self.stream[id - 1].write_all(data)?;
        Ok(data.len())
    }

    pub fn peek(&mut self, id: usize, data: &mut [u8]) -> Result<usize> {
        if id > self.stream.len() {
            bail!(format!("[{}] client {} not exist!", self.name, id));
        }
        match self.stream[id - 1].peek(data) {
            Ok(size) => Ok(size),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    Ok(0)
                } else {
                    Err(anyhow!(e))
                }
            }
        }
    }

    pub fn read(&mut self, id: usize, data: &mut [u8]) -> Result<usize> {
        if id > self.stream.len() {
            bail!(format!("[{}] client {} not exist!", self.name, id));
        }
        match self.stream[id - 1].read(data) {
            Ok(size) => Ok(size),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    Ok(0)
                } else {
                    Err(anyhow!(e))
                }
            }
        }
    }

    pub fn read_exact(&mut self, id: usize, data: &mut [u8]) -> Result<()> {
        if id > self.stream.len() {
            bail!(format!("[{}] client {} not exist!", self.name, id));
        }
        self.stream[id - 1].read_exact(data)?;
        Ok(())
    }
}

pub fn open_skt(name: &str, port: usize) -> Result<SktWrapper> {
    println!("open {} {}", name, port);
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .expect(&format!("[{}] Can not create socket @port {}", name, port));
    listener
        .set_nonblocking(true)
        .expect(&format!("[{}] Cannot set non-blocking", name));
    println!(
        "[{}] Socket @ {:?} waiting for connection!",
        name,
        listener.local_addr()?
    );
    Ok(SktWrapper {
        name: name.to_string(),
        listener,
        stream: vec![],
    })
}

#[no_mangle]
pub extern "C" fn c_skt_open(name: *const std::os::raw::c_char, port: u32) -> *mut SktWrapper {
    let name = unsafe { std::ffi::CStr::from_ptr(name) }.to_str().unwrap();
    Box::into_raw(Box::new(open_skt(name, port as usize).unwrap()))
}

#[no_mangle]
pub extern "C" fn c_skt_accept(skt: &mut SktWrapper) -> i32 {
    if let Ok(r) = skt.accept() {
        r as i32
    } else {
        -1
    }
}

fn t_skt_read<T: Sized>(skt: &mut SktWrapper, id: i32, data: *mut T) -> i32 {
    let data = unsafe { std::slice::from_raw_parts_mut(data as *mut u8, std::mem::size_of::<T>()) };
    let mut cnt = 0;
    while cnt < std::mem::size_of::<T>() {
        if let Ok(r) = skt.peek(id as usize, data) {
            if r == 0 {
                return 0;
            }
            cnt += r;
        } else {
            return -1;
        }
    }
    skt.read_exact(id as usize, data).unwrap();
    data.len() as i32
}

#[no_mangle]
pub extern "C" fn c_skt_write_u8(skt: &mut SktWrapper, id: i32, data: u8) -> i32 {
    let data: [u8; 1] = [data];
    if let Ok(r) = skt.write(id as usize, &data) {
        r as i32
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn c_skt_read_u8(skt: &mut SktWrapper, id: i32, data: *mut u8) -> i32 {
    let data = unsafe { std::slice::from_raw_parts_mut(data as *mut u8, 1) };
    match skt.read(id as usize, data) {
        Ok(r) => r as i32,
        Err(_e) => {
            // println!("{:?}", _e);
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn c_skt_write_u16(skt: &mut SktWrapper, id: i32, data: u16) -> i32 {
    if let Ok(r) = skt.write(id as usize, &data.to_le_bytes()) {
        r as i32
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn c_skt_read_u16(skt: &mut SktWrapper, id: i32, data: *mut u16) -> i32 {
    t_skt_read(skt, id, data)
}

#[no_mangle]
pub extern "C" fn c_skt_write_u32(skt: &mut SktWrapper, id: i32, data: u32) -> i32 {
    if let Ok(r) = skt.write(id as usize, &data.to_le_bytes()) {
        r as i32
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn c_skt_read_u32(skt: &mut SktWrapper, id: i32, data: *mut u32) -> i32 {
    t_skt_read(skt, id, data)
}

#[no_mangle]
pub extern "C" fn c_skt_write_u64(skt: &mut SktWrapper, id: i32, data: u64) -> i32 {
    if let Ok(r) = skt.write(id as usize, &data.to_le_bytes()) {
        r as i32
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn c_skt_read_u64(skt: &mut SktWrapper, id: i32, data: *mut u64) -> i32 {
    t_skt_read(skt, id, data)
}

#[no_mangle]
pub extern "C" fn c_skt_write_string(
    skt: &mut SktWrapper,
    id: i32,
    data: *const std::os::raw::c_char,
) -> i32 {
    let data = unsafe { std::ffi::CStr::from_ptr(data) }.to_bytes_with_nul();
    if let Ok(r) = skt.write(id as usize, data) {
        r as i32
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn c_skt_read_string(
    skt: &mut SktWrapper,
    id: i32,
    data: *mut *const std::os::raw::c_char,
) -> i32 {
    let mut buffer = vec![];
    let mut out: [u8; 1] = [1; 1];
    while out[0] != 0 {
        if let Ok(r) = skt.read(id as usize, &mut out) {
            if r == 0 || out[0] == 0 {
                buffer.push(0);
                break;
            }
        } else {
            return -1;
        }
    }
    let len = buffer.len() - 1;
    let c_str = std::ffi::CString::from_vec_with_nul(buffer).unwrap();
    let ptr = c_str.as_ptr();
    std::mem::forget(c_str);
    unsafe {
        *data = ptr;
    }
    len as i32
}

#[no_mangle]
pub extern "C" fn c_skt_close(skt: *mut SktWrapper) {
    unsafe {
        drop(Box::from_raw(skt));
    }
}
