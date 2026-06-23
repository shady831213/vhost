use super::common::{DPIShareMem, DPIShareMemParser};
use super::static_mem::{flush_static_mems, StaticMemDescriptor, StaticMemRange, StaticMemSink};
use mailbox_rs::mb_std::*;

#[derive(Default)]
struct CaptureSink {
    entries: Vec<CaptureEntry>,
}

struct CaptureEntry {
    path: String,
    width: usize,
    depth: usize,
    data: Vec<u8>,
    valid_ranges: Vec<StaticMemRange>,
}

impl StaticMemSink for CaptureSink {
    fn write_static_mem(
        &mut self,
        descriptor: &StaticMemDescriptor,
        data: &[u8],
        valid_ranges: &[StaticMemRange],
    ) -> Result<(), String> {
        self.entries.push(CaptureEntry {
            path: descriptor.path.clone(),
            width: descriptor.width,
            depth: descriptor.depth,
            data: data.to_vec(),
            valid_ranges: valid_ranges.to_vec(),
        });
        Ok(())
    }
}

#[test]
fn static_direct_flushes_instance_when_written() {
    let mem = "
radio_cim0:
    path: dut.mem
    base: 0x1000
    width: 32
    size: 16
        ";
    let mut mem = parse_mem(mem);

    assert_eq!(3, mem.write(0x1002, &[1, 2, 3]));

    let mut sink = CaptureSink::default();
    flush_static_mems(&mut sink).unwrap();

    let entry = sink
        .entries
        .iter()
        .find(|entry| entry.path == "dut.mem")
        .unwrap();
    assert_eq!(4, entry.width);
    assert_eq!(4, entry.depth);
    assert_eq!(&[0, 0, 1, 2, 3, 0, 0, 0], &entry.data[..8]);
    assert_eq!(
        vec![StaticMemRange { offset: 2, len: 3 }],
        entry.valid_ranges
    );
}

#[test]
fn static_direct_array_flushes_split_rows_when_write_crosses_row() {
    let mem = "
radio_cim0:
    path: dut.arr
    base: 0x1100
    width: 32
    size: 16
    array_dims:
        rows: 2
        cols: 2
        ";
    let mut mem = parse_mem(mem);

    assert_eq!(4, mem.write(0x1106, &[1, 2, 3, 4]));

    let mut sink = CaptureSink::default();
    flush_static_mems(&mut sink).unwrap();

    assert_static_prefix(&sink, "dut.arr[0]", &[0, 0, 0, 0, 0, 0, 1, 2]);
    assert_static_prefix(&sink, "dut.arr[1]", &[3, 4, 0, 0]);
}

#[test]
fn static_banked_flushes_each_hdl_instance_when_written() {
    let mem = "
radio_cim0:
    path: dut.sram
    base: 0x2000
    width: 128
    size: 64
    bank_width: 32
    bank_depth: 4
    banks:
      - - bank0.MEM
        - bank1.MEM
        - bank2.MEM
        - bank3.MEM
        ";
    let mut mem = parse_mem(mem);

    assert_eq!(
        16,
        mem.write(
            0x2000,
            &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
        )
    );

    let mut sink = CaptureSink::default();
    flush_static_mems(&mut sink).unwrap();

    assert_static_prefix(&sink, "dut.sram.bank0.MEM", &[1, 2, 3, 4]);
    assert_static_prefix(&sink, "dut.sram.bank1.MEM", &[5, 6, 7, 8]);
    assert_static_prefix(&sink, "dut.sram.bank2.MEM", &[9, 10, 11, 12]);
    assert_static_prefix(&sink, "dut.sram.bank3.MEM", &[13, 14, 15, 16]);
}

#[test]
fn static_banked_flushes_split_rows_when_write_crosses_bank_row() {
    let mem = "
radio_cim0:
    path: dut.vbank
    base: 0x2100
    width: 64
    size: 32
    bank_width: 64
    bank_depth: 2
    banks:
      - - row0.MEM
      - - row1.MEM
        ";
    let mut mem = parse_mem(mem);

    assert_eq!(4, mem.write(0x210e, &[1, 2, 3, 4]));

    let mut sink = CaptureSink::default();
    flush_static_mems(&mut sink).unwrap();

    assert_static_prefix(
        &sink,
        "dut.vbank.row0.MEM",
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2],
    );
    assert_static_prefix(&sink, "dut.vbank.row1.MEM", &[3, 4, 0, 0]);
}

#[test]
fn static_blackbox_parse_fails_when_no_hdl_instance_exists() {
    let mem = "
bb:
    base: 0x3000
    size: 16
        ";
    let docs = YamlLoader::load_from_str(mem).unwrap();
    let (name, value) = docs[0].as_hash().unwrap().front().unwrap();

    assert!(DPIShareMemParser
        .parse(name.as_str().unwrap(), value)
        .is_err());
}

fn parse_mem(mem: &str) -> DPIShareMem {
    let docs = YamlLoader::load_from_str(mem).unwrap();
    let (name, value) = docs[0].as_hash().unwrap().front().unwrap();
    DPIShareMemParser
        .parse(name.as_str().unwrap(), value)
        .unwrap()
}

fn assert_static_prefix(sink: &CaptureSink, path: &str, expected: &[u8]) {
    let entry = sink
        .entries
        .iter()
        .find(|entry| entry.path == path)
        .unwrap();
    assert_eq!(expected, &entry.data[..expected.len()]);
}
