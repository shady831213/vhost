use super::common::{
    DPIBankedMemHDLBuffers, DPIBankedShareMem, DPIBlackBoxShareMem, DPIDirectShareMem,
    DPIMemHDLBuffer, DPIMemHDLBuffers, DPIShareMem, InitMethod,
};
use mailbox_rs::{mb_rpcs::*, mb_std::*};

extern "C" {
    fn vhsot_bb_mem_write(base: MBPtrT, addr: MBPtrT, data: *const u8, len: usize) -> usize;
    fn vhsot_bb_mem_read(base: MBPtrT, addr: MBPtrT, data: *mut u8, len: usize) -> usize;
}

impl DPIMemHDLBuffer {
    pub fn sync_partial(&mut self) {
        panic!("vhost mem api does not support DPIMemHDLBuffer!");
    }
    pub fn flush(&self) {
        panic!("vhost mem api does not support DPIMemHDLBuffer!");
    }
    pub fn sync_all(&mut self) {
        panic!("vhost mem api does not support DPIMemHDLBuffer!");
    }
}

impl MBShareMem for DPIDirectShareMem {
    fn write(&mut self, addr: MBPtrT, data: &[u8]) -> usize {
        panic!("vhost mem api does not support DPIDirectShareMem!");
    }
    fn read(&self, addr: MBPtrT, data: &mut [u8]) -> usize {
        panic!("vhost mem api does not support DPIDirectShareMem!");
    }
}

impl MBShareMem for DPIBankedShareMem {
    fn write(&mut self, addr: MBPtrT, data: &[u8]) -> usize {
        panic!("vhost mem api does not support DPIBankedShareMem!");
    }
    fn read(&self, addr: MBPtrT, data: &mut [u8]) -> usize {
        panic!("vhost mem api does not support DPIBankedShareMem!");
    }
}

impl MBShareMem for DPIBlackBoxShareMem {
    fn write(&mut self, addr: MBPtrT, data: &[u8]) -> usize {
        unsafe { vhsot_bb_mem_write(self.base(), addr, data.as_ptr(), data.len()) }
    }
    fn read(&self, addr: MBPtrT, data: &mut [u8]) -> usize {
        unsafe { vhsot_bb_mem_read(self.base(), addr, data.as_mut_ptr(), data.len()) }
    }
}

impl DPIShareMem {
    pub(super) fn init(&mut self, _method: &InitMethod) {}
}
