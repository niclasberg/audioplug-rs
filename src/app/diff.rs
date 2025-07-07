use std::hash::Hash;

use crate::app::FxIndexSet;

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
    let mut index_map = vec![0; old.len()];

    // Need to iterate in reverse order for the removal indices to be correct
    if !old.is_empty() {
        for (i, key) in old.iter().enumerate().rev() {
            if !new.contains(key) {
                visitor(DiffOp::Remove { index: i, len: 1 });
            } else {
                index_map[i] = 1;
            }
        }

        let mut current_index = 0;
        for index in index_map.iter_mut() {
            let tmp = current_index;
            current_index += *index;
            *index = tmp;
        }
    }

    for (new_index, key) in new.iter().enumerate() {
        if let Some(old_index) = old.get_index_of(key) {
            if new_index != index_map[old_index] {
                visitor(DiffOp::Move {
                    from: old_index,
                    to: new_index,
                });
            }
        } else {
            visitor(DiffOp::Insert {
                index: new_index,
                source_index: new_index,
                len: 1,
            });
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
}
