use std::hash::Hash;

use fxhash::FxBuildHasher;

use crate::app::{FxHashSet, FxIndexSet};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DiffOp {
    Remove {
        index: usize,
        len: usize,
    },
    Replace {
        index: usize,
        source_index: usize,
    },
    Insert {
        index: usize,
        source_index: usize,
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
    // Removals:
    let mut old_to_new_index = vec![None; old.len()];

    // Need to iterate in reverse order for the removal indices to be correct
    for (i, key) in old.iter().enumerate().rev() {
        if !new.contains(key) {
            visitor(DiffOp::Remove { index: i, len: 1 });
        } else {
            old_to_new_index[i] = Some(1);
        }
    }

    // Compute indices after removals
    let mut current_index = 0;
    for index in old_to_new_index.iter_mut() {
        if let Some(index) = index {
            *index = current_index;
            current_index += 1;
        }
    }

    // Insertions:
    let mut new_to_final_index = vec![0; new.len()];
    let mut insertions = 0;
    for (i, key) in new.iter().enumerate() {
        if !old.contains(key) {
            visitor(DiffOp::Insert {
                index: i,
                source_index: i,
                len: 1,
            });
            insertions += 1;
        }
        new_to_final_index[i] = i + insertions;
    }

    // Moves:
    for (new_index, key) in new.iter().enumerate() {
        if let Some(old_index) = old.get_index_of(key) {
            let from = old_to_new_index[old_index];
            let to = new_to_final_index[new_index];
            if from != new_index && to != usize::MAX {
                visitor(DiffOp::Move {
                    from: old_index,
                    to: to,
                });
                new_to_final_index[old_index] = usize::MAX;
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
            source_index: start, // need to offset?
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
                    source_index: 0,
                    len: 3,
                }],
            ),
            TestData::new(
                b"abc",
                b"abcdef",
                &[DiffOp::Insert {
                    index: 3,
                    source_index: 3,
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
            TestData::new(b"abcdefgh", b"hgfedcba"),
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
                        for i in 0..len {
                            result.remove(index + i);
                        }
                    }
                    DiffOp::Replace {
                        index,
                        source_index,
                    } => result[index] = case.from[source_index],
                    DiffOp::Insert {
                        index,
                        source_index,
                        len,
                    } => {
                        for i in 0..len {
                            result.insert(index + i, case.to[source_index + i]);
                        }
                    }
                    DiffOp::Move { from, to } => result.swap(from, to),
                }
            });

            println!("Result: {:#?}, expected: {:#?}", result, case.to);
            println!("{:?}", ops);
            assert!(result == case.to);
        }
    }
}
