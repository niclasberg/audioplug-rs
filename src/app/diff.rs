use std::hash::Hash;

use fxhash::FxBuildHasher;

use crate::app::{FxHashMap, FxHashSet, FxIndexSet};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DiffOp {
    Remove {
        index: usize,
        len: usize,
    },
    Replace {
        index: usize,
        to_index: usize,
    },
    Insert {
        index: usize,
        // Index in the new array
        to_index: usize,
        len: usize,
    },
    Move {
        from: usize,
        to: usize,
    },
}

pub fn diff_keyed<K: Hash + Eq>(old: &FxIndexSet<K>, new: &FxIndexSet<K>) -> Vec<DiffOp> {
    let mut result = Vec::new();
    diff_keyed_with(old, new, |op| result.push(op));
    result
}

pub fn diff_keyed_with<K: Hash + Eq>(
    old: &FxIndexSet<K>,
    new: &FxIndexSet<K>,
    mut visitor: impl FnMut(DiffOp),
) {
    // Skip common prefix/suffix
    let mut start = 0;
    let mut old_end = old.len();
    let mut new_end = new.len();

    while start < old_end && start < new_end && old.get_index(start) == new.get_index(start) {
        start += 1;
    }

    while old_end > start
        && new_end > start
        && old.get_index(old_end - 1) == new.get_index(new_end - 1)
    {
        old_end -= 1;
        new_end -= 1;
    }

    let mut new_start = start;
    let mut old_start = start;
    let mut index = start;

    while old_start < old_end || new_start < new_end {
        if old_end <= old_start {
            visitor(DiffOp::Insert {
                index,
                to_index: new_start,
                len: 1,
            });
            new_start += 1;
            index += 1;
        } else if new_end <= new_start {
            visitor(DiffOp::Remove { index, len: 1 });
            old_start += 1;
        } else if old.get_index(old_start) == new.get_index(new_start) {
            // Elements matching, moving on
            old_start += 1;
            new_start += 1;
            index += 1;
        } else {
            let new_key = new.get_index(new_start).unwrap();
            let old_key = old.get_index(old_start).unwrap();

            match (old.get_index_of(new_key), new.get_index_of(old_key)) {
                (Some(from), Some(to)) => {
                    visitor(DiffOp::Move { from, to: index });
                    new_start += 1;
                    index += 1;
                }
                (None, Some(to_index)) => {
                    // The old item is not present in the new
                    visitor(DiffOp::Insert {
                        index,
                        to_index,
                        len: 1,
                    });
                    new_start += 1;
                    index += 1;
                }
                (Some(_), None) => {
                    visitor(DiffOp::Remove { index, len: 1 });
                    old_start += 1;
                    index -= 1;
                }
                (None, None) => {
                    visitor(DiffOp::Replace {
                        index,
                        to_index: old_start,
                    });
                    old_start += 1;
                    new_start += 1;
                    index += 1;
                }
            }
        }
    }
}

pub fn diff_slices<'a, T: PartialEq>(a: &'a [T], b: &'a [T]) -> Vec<DiffOp> {
    let mut ops = Vec::new();
    let mut vf = Vec::new();
    let mut vb = Vec::new();
    myers_diff_recursive(a, b, 0, 0, &mut vf, &mut vb, &mut ops);
    ops
}

fn myers_diff_recursive<'a, T: PartialEq>(
    a: &'a [T],
    b: &'a [T],
    a_offset: usize,
    b_offset: usize,
    vf: &mut Vec<usize>,
    fb: &mut Vec<usize>,
    ops: &mut Vec<DiffOp>,
) {
    let n = a.len();
    let m = b.len();

    // Trim common prefix and suffix
    let mut start = 0;
    while start < n && start < m && a[start] == b[start] {
        start += 1;
    }

    let mut a_end = n;
    let mut b_end = m;
    while a_end > start && b_end > start && a[a_end - 1] == b[b_end - 1] {
        a_end -= 1;
        b_end -= 1;
    }

    let a_is_empty = start == a_end;
    let b_is_empty = start == b_end;

    if a_is_empty && b_is_empty {
        // Nothing to do
    } else if a_is_empty {
        ops.push(DiffOp::Insert {
            index: b_offset + start,
            to_index: start, // need to offset?
            len: b_end - start,
        });
    } else if b_is_empty {
        ops.push(DiffOp::Remove {
            index: a_offset + start,
            len: a_end - start,
        });
    } else {
        let (x_mid, y_mid) = find_middle_snake(a, b, vf, fb);
    }
}

fn find_middle_snake<'a, T: PartialEq>(
    a: &'a [T],
    b: &'a [T],
    vf: &mut Vec<usize>,
    fb: &mut Vec<usize>,
) -> (usize, usize) {
    unreachable!()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_myers_diff() {
        struct TestData {
            a: Vec<u8>,
            b: Vec<u8>,
            expected: Vec<DiffOp>,
        }

        impl TestData {
            pub fn new(a: &[u8], b: &[u8], expected: &[DiffOp]) -> Self {
                Self {
                    a: a.to_owned(),
                    b: b.to_owned(),
                    expected: expected.to_vec(),
                }
            }
        }

        let cases = vec![
            TestData::new(b"abc", b"", &[DiffOp::Remove { index: 0, len: 3 }]),
            TestData::new(b"abcdef", b"abc", &[DiffOp::Remove { index: 3, len: 3 }]),
            TestData::new(b"abcdef", b"def", &[DiffOp::Remove { index: 0, len: 3 }]),
            TestData::new(
                b"",
                b"abc",
                &[DiffOp::Insert {
                    index: 0,
                    to_index: 0,
                    len: 3,
                }],
            ),
            TestData::new(
                b"abc",
                b"abcdef",
                &[DiffOp::Insert {
                    index: 3,
                    to_index: 3,
                    len: 3,
                }],
            ),
        ];

        for case in cases {
            let result = diff_slices(&case.a, &case.b);
            assert_eq!(result, case.expected);
        }
    }

    #[test]
    fn test_keyed_diff() {
        struct TestData {
            from: Vec<u8>,
            to: Vec<u8>,
        }

        impl TestData {
            pub fn new(a: &[u8], b: &[u8]) -> Self {
                Self {
                    from: a.to_owned(),
                    to: b.to_owned(),
                }
            }
        }

        let cases = vec![
            TestData::new(b"", b""),
            TestData::new(b"", b"abc"),
            TestData::new(b"abc", b""),
            TestData::new(b"abc", b"def"),
            TestData::new(b"abcdefgh", b"hgfedcba"),
            TestData::new(b"abc", b"cfdgea"),
            TestData::new(b"abcdefgh", b"ahbgcfde"),
        ];

        for case in cases {
            let mut result = case.from.clone();
            let from_indices: FxIndexSet<_> = case.from.iter().collect();
            let to_indices: FxIndexSet<_> = case.to.iter().collect();
            let mut ops = Vec::new();

            diff_keyed_with(&from_indices, &to_indices, |op| {
                ops.push(op);
                match op {
                    DiffOp::Remove { index, len } => {
                        result.drain(index..(index + len));
                    }
                    DiffOp::Replace {
                        index,
                        to_index: source_index,
                    } => result[index] = case.to[source_index],
                    DiffOp::Insert {
                        index,
                        to_index,
                        len,
                    } => {
                        result.splice(
                            index..index,
                            case.to[to_index..(to_index + len)].iter().copied(),
                        );
                    }
                    DiffOp::Move { from, to } => {
                        let item = result.remove(from);
                        result.insert(to, item);
                    }
                }
            });

            println!("Result: {:#?}, expected: {:#?}", result, case.to);
            println!("{:?}", ops);
            assert!(result == case.to);
        }
    }
}
