#![feature(io_error_more)]
#![feature(map_try_insert)]
pub extern crate mailbox_rs;
use mailbox_rs::{mb_rpcs::MBPtrT, mb_std::*};
use std::env;
pub mod mem;
use mem::*;
pub mod rpcs;
pub mod sockets;
use std::collections::HashMap;
use std::future::Future;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

pub type MemSpaces = HashMap<String, Arc<Mutex<DPIShareMemSpace>>>;
pub type FileSyses = HashMap<String, Box<dyn MBFileOpener>>;
pub type VHostMb = MBChannelShareMemSys<DPIShareMemSpace>;

pub struct MailboxInitPaths<P1: AsRef<Path>, P2: AsRef<Path>> {
    pub mem_cfg_file: P1,
    pub mailbox_cfg_file: P2,
}

pub fn mailbox_init_with_paths<P1: AsRef<Path>, P2: AsRef<Path>>(
    paths: MailboxInitPaths<P1, P2>,
) -> Result<VHostMb, String> {
    let mailbox_cfg =
        std::fs::read_to_string(paths.mailbox_cfg_file.as_ref()).map_err(|e| e.to_string())?;
    mailbox_init_with_config(paths.mem_cfg_file, &mailbox_cfg)
}

pub fn mailbox_init_with_config<P: AsRef<Path>>(
    mem_cfg_file: P,
    mailbox_cfg: &str,
) -> Result<VHostMb, String> {
    #[cfg(feature = "mem_static")]
    clear_static_mems();
    let mem_cfg_file = mem_cfg_file
        .as_ref()
        .to_str()
        .ok_or_else(|| format!("invalid mem cfg path {}", mem_cfg_file.as_ref().display()))?;
    let spaces = MBShareMemSpaceBuilder::<DPIShareMem, DPIShareMemParser>::new(mem_cfg_file)?
        .build_shared()?
        .build_spaces()?;
    MBChannelShareMemBuilder::<DPIShareMemSpace>::from_str(mailbox_cfg, spaces)?
        .cfg_channels()
        .map(MBChannelShareMemBuilder::build)
}

pub fn mailbox_init<
    F1: Fn(&MemSpaces) -> Result<(), String>,
    F2: FnMut(&mut FileSyses) -> Result<(), String>,
    F3: FnMut(&mut FileSyses) -> Result<(), String>,
>(
    spaces_cb: F1,
    special_fs_cb: F2,
    virtual_fs_cb: F3,
) -> MBChannelShareMemSys<DPIShareMemSpace> {
    #[cfg(feature = "mem_static")]
    clear_static_mems();
    let spaces = {
        MBShareMemSpaceBuilder::<DPIShareMem, DPIShareMemParser>::new(
            &env::var("MEM_CFG_FILE").unwrap(),
        )
        .unwrap()
        .build_shared()
        .unwrap()
        .build_spaces()
        .unwrap()
    };
    spaces_cb(&spaces).unwrap();
    MBChannelShareMemBuilder::<DPIShareMemSpace>::new(
        &env::var("MAILBOX_CFG_FILE").unwrap(),
        spaces,
    )
    .unwrap()
    .cfg_channels()
    .unwrap()
    .fs_with_special_and_virtual(
        &env::var("MAILBOX_FS_ROOT").unwrap(),
        special_fs_cb,
        virtual_fs_cb,
    )
    .unwrap()
    .build()
}

fn mb_tick() -> bool {
    extern "C" {
        fn mb_step();
    }
    unsafe {
        mb_step();
    }
    false
}

#[no_mangle]
unsafe extern "C" fn __mb_call(
    ch_name: *const std::os::raw::c_char,
    method: *const std::os::raw::c_char,
    arg_len: u32,
    args: *const MBPtrT,
    status: &mut u32,
) -> MBPtrT {
    extern "C" {
        fn mb_sv_call(
            ch_name: *const std::os::raw::c_char,
            method: *const std::os::raw::c_char,
            arg_len: u32,
            args: *const MBPtrT,
            status: &mut u32,
        ) -> MBPtrT;
    }
    #[cfg(feature = "pyo3")]
    {
        if let Ok((finished, result)) = rpcs::py_calls::poll_py_call(ch_name, method, arg_len, args)
        {
            *status = finished as u32;
            return result;
        }
    }
    mb_sv_call(ch_name, method, arg_len, args, status)
}

pub fn mb_server_run<'a, B: Fn() -> bool + 'a, F: Fn(&MBSMServer<DPIShareMemSpace>)>(
    mb: &'a VHostMb,
    server_cb: F,
    waker_breaker: B,
) -> (
    impl Future<Output = ()> + use<'a, B, F>,
    Vec<impl Future<Output = (String, u32)> + use<'a, B, F> + std::marker::Unpin>,
) {
    (
        mb.wake(move || {
            mb_tick();
            waker_breaker()
        }),
        mb.serve(move |server| server_cb(server)),
    )
}
