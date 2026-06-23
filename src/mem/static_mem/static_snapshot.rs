use std::collections::BTreeMap;

use super::StaticMemDescriptor;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticMemRange {
    pub offset: usize,
    pub len: usize,
}

pub(super) fn dense_data(
    descriptor: &StaticMemDescriptor,
    sparse: &BTreeMap<usize, u8>,
) -> Vec<u8> {
    let mut data = vec![0; descriptor.width * descriptor.depth];
    for (offset, byte) in sparse {
        if let Some(slot) = data.get_mut(*offset) {
            *slot = *byte;
        }
    }
    data
}

pub(super) fn collect_valid_ranges(data: &BTreeMap<usize, u8>) -> Vec<StaticMemRange> {
    let mut ranges = Vec::new();
    let mut start_and_end = None;
    for offset in data.keys().copied() {
        match start_and_end {
            None => start_and_end = Some((offset, offset + 1)),
            Some((range_start, range_end)) if offset == range_end => {
                start_and_end = Some((range_start, offset + 1));
            }
            Some((range_start, range_end)) => {
                ranges.push(StaticMemRange {
                    offset: range_start,
                    len: range_end - range_start,
                });
                start_and_end = Some((offset, offset + 1));
            }
        }
    }
    if let Some((range_start, range_end)) = start_and_end {
        ranges.push(StaticMemRange {
            offset: range_start,
            len: range_end - range_start,
        });
    }
    ranges
}
