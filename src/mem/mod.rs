#[cfg(any(
    all(feature = "mem_uvm", feature = "mem_api"),
    all(feature = "mem_uvm", feature = "mem_static"),
    all(feature = "mem_api", feature = "mem_static")
))]
compile_error!("features mem_uvm, mem_api, and mem_static are mutually exclusive");

#[cfg(feature = "mem_uvm")]
mod uvm_mem;

#[cfg(all(not(feature = "mem_uvm"), feature = "mem_api"))]
mod mem_api;

#[cfg(all(
    feature = "mem_static",
    not(any(feature = "mem_uvm", feature = "mem_api"))
))]
mod static_mem;

#[cfg(all(
    test,
    feature = "mem_static",
    not(any(feature = "mem_uvm", feature = "mem_api"))
))]
mod static_mem_tests;

#[cfg(all(
    feature = "mem_static",
    not(any(feature = "mem_uvm", feature = "mem_api"))
))]
pub use static_mem::*;

mod common;
pub use common::*;

#[cfg(not(any(feature = "mem_uvm", feature = "mem_api", feature = "mem_static")))]
mod default;
