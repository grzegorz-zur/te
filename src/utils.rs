use std::cmp::min;
use std::ops::Range;

pub fn sub(string: &str, range: Range<usize>) -> &str {
    match string.get(range.start..min(string.len(), range.end)) {
        None => "",
        Some(s) => s,
    }
}
