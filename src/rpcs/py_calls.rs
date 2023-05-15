use mailbox_rs::mb_rpcs::*;
use pyo3::{intern, prelude::*, types::IntoPyDict};
pub fn poll_py_call(
    ch_name: *const std::os::raw::c_char,
    method: *const std::os::raw::c_char,
    arg_len: u32,
    args: *const MBPtrT,
) -> Result<(MBCallStatus, MBPtrT), String> {
    let ch_name_str = unsafe { std::ffi::CStr::from_ptr(ch_name) }
        .to_str()
        .unwrap();
    let method_str = unsafe { std::ffi::CStr::from_ptr(method) }
        .to_str()
        .unwrap();
    let args = unsafe { std::slice::from_raw_parts(args, arg_len as usize) };

    Python::with_gil(|py| -> Result<(MBCallStatus, MBPtrT), String> {
        let locals = [
            (intern!(py, "func_name"), method_str),
            (intern!(py, "id"), ch_name_str),
        ]
        .into_py_dict(py);
        locals.set_item(intern!(py, "args"), args).unwrap();
        py.run(
            r#"
from pymb_rpcs import poll
(exists, finished, ret) = poll(func_name, id, *args)
        "#,
            None,
            Some(&locals),
        )
        .map_err(|e| {
            e.print(py);
            format!("{}: {}", e.get_type(py), e.value(py))
        })
        .unwrap();
        let exists: bool = locals
            .get_item(intern!(py, "exists"))
            .unwrap()
            .extract()
            .unwrap();
        if !exists {
            return Err(format!("{} does not exist!", method_str));
        }
        let finished: bool = locals
            .get_item(intern!(py, "finished"))
            .unwrap()
            .extract()
            .unwrap();
        if finished {
            let result: MBPtrT = locals
                .get_item(intern!(py, "ret"))
                .unwrap()
                .extract()
                .unwrap();
            Ok((MBCallStatus::Ready, result))
        } else {
            Ok((MBCallStatus::Pending, 0))
        }
    })
}

#[allow(dead_code)]
pub fn py_tick() {
    Python::with_gil(|py| -> Result<(), String> {
        let err_handle = |e: PyErr| -> String {
            e.print(py);
            format!("{}: {}", e.get_type(py), e.value(py))
        };
        let asyncio = py.import(intern!(py, "asyncio")).map_err(err_handle)?;
        let event_loop = asyncio
            .call_method0(intern!(py, "get_event_loop"))
            .map_err(err_handle)?;
        event_loop
            .call_method0(intern!(py, "stop"))
            .map_err(err_handle)?;
        event_loop
            .call_method0(intern!(py, "run_forever"))
            .map_err(err_handle)?;
        Ok(())
    })
    .unwrap();
}

#[allow(dead_code)]
pub fn py_calls_init() {
    if Python::with_gil(|py| -> Result<(), String> {
        let err_handle = |e: PyErr| -> String {
            e.print(py);
            format!("{}: {}", e.get_type(py), e.value(py))
        };
        let init = py
            .import(intern!(py, "mbpy_calls_init"))
            .map_err(err_handle)?;
        init.call_method0(intern!(py, "mbpy_calls_init"))
            .map_err(err_handle)?;
        Ok(())
    })
    .is_err()
    {
        println!("Warning! Can not find mbpy_calls init func \"mbpy_calls_init()\" in module \"mbpy_calls_init\"!")
    } else {
        println!("Call \"mbpy_calls_init()\" in module \"mbpy_calls_init\" successfully!")
    }
}
