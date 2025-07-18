use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use rustc_hash::FxBuildHasher;

use crate::app::{FxHashMap, FxHashSet, FxIndexSet};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DiffOp<'a, T> {
    Remove {
        index: usize,
        len: usize,
    },
    Replace {
        index: usize,
        value: &'a T,
    },
    Insert {
        index: usize,
        // Index in the new array
        values: &'a [T],
    },
    Move {
        from: usize,
        to: usize,
    },
}

pub fn diff_keyed<'a, K: Hash + Eq, T: 'a>(
    old: &FxIndexSet<K>,
    new: &FxIndexSet<K>,
    new_data: &'a [T],
) -> Vec<DiffOp<'a, T>> {
    let mut result = Vec::new();
    diff_keyed_with(old, new, new_data, |op| result.push(op));
    result
}

/// Computes the diff operations needed to transform from `old`` to `new`
/// The old and new index sets contains hashed keys, and the
/// new_data the original data that `new` was computed from.
///
/// The diff algorithm is adapted from
/// https://github.com/frejs/fre/blob/master/src/reconcile.ts
pub fn diff_keyed_with<'a, K: Hash + Eq, T: 'a>(
    old: &FxIndexSet<K>,
    new: &FxIndexSet<K>,
    new_data: &'a [T],
    mut visitor: impl FnMut(DiffOp<'a, T>),
) {
    let mut start = 0;
    let mut old_end = old.len();
    let mut new_end = new.len();

    // Skip common prefix
    while start < old_end && start < new_end && old.get_index(start) == new.get_index(start) {
        start += 1;
    }

    // Skip common suffix
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
    let mut is_moved = vec![false; old.len()];

    while old_start < old_end || new_start < new_end {
        if old_end <= old_start {
            // No more elements to process in old, insert the remaining entries
            // from new
            visitor(DiffOp::Insert {
                index,
                values: &new_data[new_start..new_end],
            });
            break;
        } else if is_moved[old_start] {
            old_start += 1;
        } else if new_end <= new_start {
            visitor(DiffOp::Remove { index, len: 1 });
            old_start += 1;
        } else if new.get_index(new_start) == old.get_index(old_start) {
            // Elements matching, moving on
            old_start += 1;
            new_start += 1;
            index += 1;
        } else {
            let new_key = new.get_index(new_start).unwrap();
            let old_key = old.get_index(old_start).unwrap();
            match (old.get_index_of(new_key), new.get_index_of(old_key)) {
                (Some(from), Some(_)) => {
                    let offset = is_moved
                        .iter()
                        .skip(old_start)
                        .take(from - old_start)
                        .fold(0, |acc, moved| acc + if *moved { 0 } else { 1 });
                    visitor(DiffOp::Move {
                        from: index + offset,
                        to: index,
                    });
                    new_start += 1;
                    index += 1;
                    is_moved[from] = true;
                }
                (None, Some(_)) => {
                    visitor(DiffOp::Remove { index, len: 1 });
                    old_start += 1;
                }
                (Some(_), None) => {
                    visitor(DiffOp::Insert {
                        index,
                        values: std::slice::from_ref(&new_data[new_start]),
                    });
                    new_start += 1;
                    index += 1;
                }
                (None, None) => {
                    visitor(DiffOp::Replace {
                        index,
                        value: &new_data[new_start],
                    });
                    old_start += 1;
                    new_start += 1;
                    index += 1;
                }
            }
        }
    }
}

/// Computes the diff operations needed for transforming `a` to `b`
///
/// The implementation uses the Myer's diff algorithm (with a divide and conquer approach
/// inspired by https://github.com/mitsuhiko/similar/blob/main/src/algorithms/myers.rs)
pub fn diff_slices<'a, T: PartialEq>(a: &[T], b: &'a [T]) -> Vec<DiffOp<'a, T>> {
    let mut ops = Vec::new();
    let mut vf = Vec::new();
    let mut vb = Vec::new();
    myers_diff_recursive(a, b, 0, 0, &mut vf, &mut vb, &mut ops);
    ops
}

fn myers_diff_recursive<'a, T: PartialEq + 'a>(
    a: &[T],
    b: &'a [T],
    a_offset: usize,
    b_offset: usize,
    vf: &mut Vec<usize>,
    fb: &mut Vec<usize>,
    ops: &mut Vec<DiffOp<'a, T>>,
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
            values: &b[start..b_end],
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
            expected: Vec<DiffOp<'static, u8>>,
        }

        impl TestData {
            pub fn new(a: &[u8], b: &[u8], expected: &[DiffOp<'static, u8>]) -> Self {
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
                    values: b"abc",
                }],
            ),
            TestData::new(
                b"abc",
                b"abcdef",
                &[DiffOp::Insert {
                    index: 3,
                    values: b"def",
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
            TestData::new(b"abc", b"cba"),
            TestData::new(b"abc", b"cfdgea"),
            TestData::new(b"abcdefgh", b"hgfedcba"),
            TestData::new(b"abcdefgh", b"efghabcd"),
            TestData::new(b"abcdefgh", b"ahbgcfde"),
            TestData::new(b"abcdefgh", b"ahbgxyzcfde"),
            TestData::new(b"abcdexyzfgh", b"ahbgcfde"),
        ];

        for case in cases {
            let mut result = case.from.clone();
            let from_indices: FxIndexSet<_> = case.from.iter().collect();
            let to_indices: FxIndexSet<_> = case.to.iter().collect();

            diff_keyed_with(&from_indices, &to_indices, &case.to, |op| {
                println!("{:?}", op);
                match op {
                    DiffOp::Remove { index, len } => {
                        result.drain(index..(index + len));
                    }
                    DiffOp::Replace { index, value } => result[index] = *value,
                    DiffOp::Insert { index, values } => {
                        result.splice(index..index, values.iter().copied());
                    }
                    DiffOp::Move { from, to } => {
                        let item = result.remove(from);
                        result.insert(to, item);
                    }
                }
            });

            println!("Result: {:#?}, expected: {:#?}", result, case.to);
            assert!(result == case.to);
        }
    }
}
