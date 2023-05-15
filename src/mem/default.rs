use super::common::{
    DPIBankedMemHDLBuffers, DPIBankedShareMem, DPIBlackBoxShareMem, DPIDirectShareMem,
    DPIMemHDLBuffer, DPIMemHDLBuffers, DPIShareMem, InitMethod,
};
use mailbox_rs::{mb_rpcs::*, mb_std::*};

impl DPIMemHDLBuffer {
    pub fn sync_partial(&mut self) {
        if self.head_unaligned {
            println!("{} sync {} to buffer [0:width]", self.path(), self.head_idx)
        }
        if self.tail_unaligned {
            println!(
                "{} sync {} to buffer [end-width:end]",
                self.path(),
                self.tail_idx
            )
        }
    }
    pub fn flush(&self) {
        println!(
            "{} write {} data to [{}:{}]",
            self.path(),
            self.buffer.len(),
            self.head_idx,
            self.tail_idx
        )
    }
    pub fn sync_all(&mut self) {
        println!(
            "{} sync [{}:{}] to buffer",
            self.path(),
            self.head_idx,
            self.tail_idx
        )
    }
}

impl MBShareMem for DPIDirectShareMem {
    fn write(&mut self, addr: MBPtrT, data: &[u8]) -> usize {
        let addr = self.offset(addr);
        let mut buffers = DPIMemHDLBuffers::new(&self.array, addr, data.len());
        buffers.sync_partial();
        for (i, d) in data.iter().enumerate() {
            let offset = addr + i;
            let buffer = buffers.get_buffer_mut(offset);
            buffer.set_data(offset, *d);
        }
        buffers.flush();
        data.len()
    }
    fn read(&self, addr: MBPtrT, data: &mut [u8]) -> usize {
        let addr = self.offset(addr);
        let mut buffers = DPIMemHDLBuffers::new(&self.array, addr, data.len());
        buffers.sync_all();
        for (i, d) in data.iter_mut().enumerate() {
            let offset = addr as usize + i;
            let buffer = buffers.get_buffer(offset);
            *d = buffer.get_data(offset);
        }
        data.len()
    }
}

impl MBShareMem for DPIBankedShareMem {
    fn write(&mut self, addr: MBPtrT, data: &[u8]) -> usize {
        let addr = self.offset(addr);
        let mut buffers = DPIBankedMemHDLBuffers::new(self, addr, data.len());
        buffers.sync();
        for (i, d) in data.iter().enumerate() {
            let offset = addr + i;
            let array = buffers.get_array_mut(self.row(offset), self.col(offset));
            let bank_offset = self.bank_offset(offset);
            let buffer = array.get_buffer_mut(bank_offset);
            buffer.set_data(bank_offset, *d);
        }
        buffers.flush();
        data.len()
    }
    fn read(&self, addr: MBPtrT, data: &mut [u8]) -> usize {
        let addr = self.offset(addr);
        let mut buffers = DPIBankedMemHDLBuffers::new(self, addr, data.len());
        buffers.sync();
        for (i, d) in data.iter_mut().enumerate() {
            let offset = addr + i;
            let array = buffers.get_array(self.row(offset), self.col(offset));
            let bank_offset = self.bank_offset(offset);
            let buffer = array.get_buffer(bank_offset);
            *d = buffer.get_data(bank_offset);
        }
        data.len()
    }
}

impl MBShareMem for DPIBlackBoxShareMem {
    fn write(&mut self, _addr: MBPtrT, data: &[u8]) -> usize {
        data.len()
    }
    fn read(&self, _addr: MBPtrT, data: &mut [u8]) -> usize {
        data.len()
    }
}

impl DPIShareMem {
    pub(super) fn init(&mut self, _method: &InitMethod) {}
}
