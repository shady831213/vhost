use super::common::{
    DPIBankedShareMem, DPIBlackBoxShareMem, DPIDirectShareMem, DPIMemArray, DPIMemHDLBuffer,
    DPIShareMem, InitMethod,
};
use lazy_static::lazy_static;
use mailbox_rs::{mb_rpcs::*, mb_std::*};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticMemDescriptor {
    pub path: String,
    pub width: usize,
    pub depth: usize,
}

pub trait StaticMemSink {
    fn write_static_mem(
        &mut self,
        descriptor: &StaticMemDescriptor,
        data: &[u8],
    ) -> Result<(), String>;
}

#[derive(Clone, Debug)]
struct StaticMemBuffer {
    descriptor: StaticMemDescriptor,
    data: Vec<u8>,
}

lazy_static! {
    static ref STATIC_MEMS: Mutex<Vec<StaticMemBuffer>> = Mutex::new(vec![]);
}

pub fn clear_static_mems() {
    if let Ok(mut mems) = STATIC_MEMS.lock() {
        mems.clear();
    }
}

pub fn flush_static_mems<S: StaticMemSink>(sink: &mut S) -> Result<(), String> {
    let snapshots = STATIC_MEMS.lock().map_err(|e| e.to_string())?.clone();
    for snapshot in snapshots.iter() {
        sink.write_static_mem(&snapshot.descriptor, &snapshot.data)?;
    }
    Ok(())
}

fn register_instance(path: String, width: usize, depth: usize) {
    if let Ok(mut mems) = STATIC_MEMS.lock() {
        if mems.iter().any(|mem| {
            mem.descriptor.path == path
                && mem.descriptor.width == width
                && mem.descriptor.depth == depth
        }) {
            return;
        }
        mems.push(StaticMemBuffer {
            descriptor: StaticMemDescriptor { path, width, depth },
            data: vec![0; width * depth],
        });
    }
}

fn register_array(array: &Arc<DPIMemArray>) {
    for row in 0..array.row_count() {
        register_instance(array.array_hdl_path(row), array.width, array.row_depth());
    }
}

fn write_instance_locked(
    mems: &mut [StaticMemBuffer],
    path: &str,
    width: usize,
    depth: usize,
    offset: usize,
    byte: u8,
) -> bool {
    let Some(mem) = mems.iter_mut().find(|mem| {
        mem.descriptor.path == path
            && mem.descriptor.width == width
            && mem.descriptor.depth == depth
    }) else {
        return false;
    };
    if offset >= mem.data.len() {
        return false;
    }
    mem.data[offset] = byte;
    true
}

fn read_instance_locked(
    mems: &[StaticMemBuffer],
    path: &str,
    width: usize,
    depth: usize,
    offset: usize,
) -> Option<u8> {
    let mem = mems.iter().find(|mem| {
        mem.descriptor.path == path
            && mem.descriptor.width == width
            && mem.descriptor.depth == depth
    })?;
    mem.data.get(offset).copied()
}

fn array_offset(array: &DPIMemArray, offset: usize) -> (String, usize) {
    let row = array.row(offset);
    let local = array.col(offset) * array.width + array.idx_byte(offset);
    (array.array_hdl_path(row), local)
}

fn write_array(array: &Arc<DPIMemArray>, offset: usize, data: &[u8]) -> usize {
    register_array(array);
    let depth = array.row_depth();
    let Ok(mut mems) = STATIC_MEMS.lock() else {
        return 0;
    };
    for (i, byte) in data.iter().enumerate() {
        let (path, local) = array_offset(array, offset + i);
        if !write_instance_locked(&mut mems, &path, array.width, depth, local, *byte) {
            return i;
        }
    }
    data.len()
}

fn read_array(array: &Arc<DPIMemArray>, offset: usize, data: &mut [u8]) -> usize {
    register_array(array);
    let depth = array.row_depth();
    let Ok(mems) = STATIC_MEMS.lock() else {
        return 0;
    };
    for (i, byte) in data.iter_mut().enumerate() {
        let (path, local) = array_offset(array, offset + i);
        let Some(value) = read_instance_locked(&mems, &path, array.width, depth, local) else {
            return i;
        };
        *byte = value;
    }
    data.len()
}

fn register_banked(mem: &DPIBankedShareMem) {
    for row in mem.banks.iter() {
        for array in row.iter() {
            register_array(array);
        }
    }
}

impl DPIMemHDLBuffer {
    pub fn sync_partial(&mut self) {}
    pub fn flush(&self) {}
    pub fn sync_all(&mut self) {}
}

impl MBShareMem for DPIBlackBoxShareMem {
    fn write(&mut self, _addr: MBPtrT, _data: &[u8]) -> usize {
        panic!("mem_static does not support blackbox memory!")
    }
    fn read(&self, _addr: MBPtrT, _data: &mut [u8]) -> usize {
        panic!("mem_static does not support blackbox memory!")
    }
}

impl MBShareMem for DPIDirectShareMem {
    fn write(&mut self, addr: MBPtrT, data: &[u8]) -> usize {
        write_array(&self.array, self.offset(addr), data)
    }
    fn read(&self, addr: MBPtrT, data: &mut [u8]) -> usize {
        read_array(&self.array, self.offset(addr), data)
    }
}

impl MBShareMem for DPIBankedShareMem {
    fn write(&mut self, addr: MBPtrT, data: &[u8]) -> usize {
        register_banked(self);
        let base_offset = self.offset(addr);
        let Ok(mut mems) = STATIC_MEMS.lock() else {
            return 0;
        };
        for (i, byte) in data.iter().enumerate() {
            let offset = base_offset + i;
            let array = &self.banks[self.row(offset)][self.col(offset)];
            let bank_offset = self.bank_offset(offset);
            let (path, local) = array_offset(array, bank_offset);
            if !write_instance_locked(
                &mut mems,
                &path,
                array.width,
                array.row_depth(),
                local,
                *byte,
            ) {
                return i;
            }
        }
        data.len()
    }
    fn read(&self, addr: MBPtrT, data: &mut [u8]) -> usize {
        register_banked(self);
        let base_offset = self.offset(addr);
        let Ok(mems) = STATIC_MEMS.lock() else {
            return 0;
        };
        for (i, byte) in data.iter_mut().enumerate() {
            let offset = base_offset + i;
            let array = &self.banks[self.row(offset)][self.col(offset)];
            let bank_offset = self.bank_offset(offset);
            let (path, local) = array_offset(array, bank_offset);
            let Some(value) =
                read_instance_locked(&mems, &path, array.width, array.row_depth(), local)
            else {
                return i;
            };
            *byte = value;
        }
        data.len()
    }
}

impl DPIShareMem {
    pub(super) fn init(&mut self, _method: &InitMethod) {
        match self {
            DPIShareMem::BlackBox(_) => {}
            DPIShareMem::Direct(mem) => register_array(&mem.array),
            DPIShareMem::Banked(mem) => register_banked(mem),
        }
    }
}
