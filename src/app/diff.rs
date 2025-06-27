use std::collections::HashMap;
use std::hash::Hash;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DiffOp<'a, T> {
    Remove { index: usize, len: usize },
    Change { index: usize, new_value: &'a T },
    Insert { index: usize, values: &'a [T] },
    Move { old_index: usize, new_index: usize },
}

fn diff_keyed<K: Hash + PartialEq>(a: &HashMap<K, usize>, b: &HashMap<K, usize>) {
    if a.is_empty() && b.is_empty() {}
}

pub fn diff_slices<'a, T: PartialEq>(a: &'a [T], b: &'a [T]) -> Vec<DiffOp<'a, T>> {
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
}
