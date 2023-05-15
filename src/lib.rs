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
use std::sync::Arc;
use std::sync::Mutex;

pub type MemSpaces = HashMap<String, Arc<Mutex<DPIShareMemSpace>>>;
pub type FileSyses = HashMap<String, Box<dyn MBFileOpener>>;
pub type VHostMb = MBChannelShareMemSys<DPIShareMemSpace>;

pub fn mailbox_init<
    F1: Fn(&MemSpaces) -> Result<(), String>,
    F2: FnMut(&mut FileSyses) -> Result<(), String>,
    F3: FnMut(&mut FileSyses) -> Result<(), String>,
>(
    spaces_cb: F1,
    special_fs_cb: F2,
    virtual_fs_cb: F3,
) -> MBChannelShareMemSys<DPIShareMemSpace> {
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

fn mb_tick() {
    extern "C" {
        fn mb_step();
    }
    unsafe {
        mb_step();
    }
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

pub fn mb_server_run<F: Fn(), F1: Fn(&MBSMServer<DPIShareMemSpace>)>(
    mb: &VHostMb,
    prepare: F,
    server_cb: F1,
) {
    use mailbox_rs::mb_std::futures::future::join;
    let w = mb.wake(mb_tick);
    let s = mb.serve(server_cb);
    prepare();
    println!("mb_server_run start!");
    async_std::task::block_on(async move {
        join(w, s).await;
    });
}

pub fn mb_server_run_async<F: Fn(), F1: Fn(&MBSMServer<DPIShareMemSpace>)>(
    mb: &'static VHostMb,
    prepare: F,
    server_cb: F1,
) {
    use mailbox_rs::mb_std::futures::future::join;
    let w = mb.wake(mb_tick);
    let s = mb.serve(server_cb);
    prepare();
    println!("mb_server_run start!");
    async_std::task::spawn(async move {
        join(w, s).await;
    });
}
//__mb_exit(ch_name: *const std::os::raw::c_char, code: u32) should be impl project by project
