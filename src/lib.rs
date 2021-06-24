use similar::{Algorithm, ChangeTag, TextDiff};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn line_diff(old_text: &str, new_text: &str) -> Vec<u8> {
    let result = diff(old_text, new_text);

    // turn sensible struct vec into something that can be passed across
    // the wasm boundary
    let mut magic_numbers: std::vec::Vec<u8> = Vec::new();

    for d in result.iter() {
        let start_line_bytes = transform_u32_to_array_of_u8(d.start_line);
        magic_numbers.extend(start_line_bytes);
        let end_line_bytes = transform_u32_to_array_of_u8(d.end_line);
        magic_numbers.extend(end_line_bytes);
        magic_numbers.push(d.kind as u8);
    }

    magic_numbers
}

fn transform_u32_to_array_of_u8(x: u32) -> [u8; 4] {
    let b1: u8 = ((x >> 24) & 0xff) as u8;
    let b2: u8 = ((x >> 16) & 0xff) as u8;
    let b3: u8 = ((x >> 8) & 0xff) as u8;
    let b4: u8 = (x & 0xff) as u8;
    return [b1, b2, b3, b4];
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum DiffKind {
    Add = 1,
    Delete = 2,
    Modify = 3,
}

#[derive(Debug, PartialEq)]
struct Diff {
    start_line: u32,
    end_line: u32,
    kind: DiffKind,
}

fn diff(old_text: &str, new_text: &str) -> Vec<Diff> {
    let diff = TextDiff::configure()
        .algorithm(Algorithm::Patience)
        .diff_lines(old_text, new_text);

    let mut line_new_text = 1;
    let mut active_delete_line_count = 0;

    let mut diff_vec: std::vec::Vec<Diff> = Vec::new();

    // Process all changes into "new document" line numbers
    // with deletes collapsed into single "carats" and delete/inserts collapsed into "modifes"
    for change in diff.iter_all_changes() {
        if matches!(change.tag(), ChangeTag::Equal) && active_delete_line_count > 0 {
            active_delete_line_count = 0;
            diff_vec.push(Diff {
                start_line: line_new_text,
                end_line: line_new_text,
                kind: DiffKind::Delete,
            })
        }

        if matches!(change.tag(), ChangeTag::Delete) {
            active_delete_line_count += 1;
        }

        if matches!(change.tag(), ChangeTag::Insert) {
            if active_delete_line_count > 0 {
                active_delete_line_count -= 1;
                diff_vec.push(Diff {
                    start_line: line_new_text,
                    end_line: line_new_text,
                    kind: DiffKind::Modify,
                });
            } else {
                diff_vec.push(Diff {
                    start_line: line_new_text,
                    end_line: line_new_text,
                    kind: DiffKind::Add,
                });
            }
        }

        if matches!(change.tag(), ChangeTag::Equal) || matches!(change.tag(), ChangeTag::Insert) {
            line_new_text += 1;
        }
    }

    if active_delete_line_count > 0 {
        diff_vec.push(Diff {
            start_line: line_new_text,
            end_line: line_new_text,
            kind: DiffKind::Delete,
        })
    }

    // horrible way to collapse adjacent changes into multi-line bars
    // xxx: surely a more idiomatic rust way, or an in-place algorithm?
    let mut merged_vec: std::vec::Vec<Diff> = Vec::new();

    let mut skip = 0;
    for i in 0..diff_vec.len() {
        if i < skip {
            continue;
        }
        let mut current = Diff {
            start_line: diff_vec[i].start_line,
            end_line: diff_vec[i].end_line,
            kind: diff_vec[i].kind,
        };
        for j in i + 1..diff_vec.len() {
            if diff_vec[j].kind == current.kind && diff_vec[j].start_line == current.end_line + 1 {
                current.end_line = diff_vec[j].end_line;
                skip += 1;
            } else {
                break;
            }
        }
        merged_vec.push(current);
        skip += 1;
    }

    merged_vec
}

#[cfg(test)]
mod tests {
    use crate::diff;
    use crate::line_diff;
    use crate::Diff;
    use crate::DiffKind;

    fn vec_compare(va: std::vec::Vec<Diff>, vb: std::vec::Vec<Diff>) -> bool {
        (va.len() == vb.len()) &&  // zip stops at the shortest
         va.iter()
           .zip(vb)
           .all(|(a,b)| *a == b)
    }

    #[test]
    fn no_changes() {
        let out = diff("hello, world\n2\n3\n4\n", "hello, world\n2\n3\n4\n");
        let expected = vec![];
        assert_eq!(vec_compare(out, expected), true);
    }

    #[test]
    fn single_add() {
        let out = diff("", "hello, world\n");
        let expected = vec![Diff {
            kind: DiffKind::Add,
            start_line: 1,
            end_line: 1,
        }];
        assert_eq!(vec_compare(out, expected), true);
    }

    #[test]
    fn single_delete() {
        let out = diff("hello, world\n", "");
        let expected = vec![Diff {
            kind: DiffKind::Delete,
            start_line: 1,
            end_line: 1,
        }];
        assert_eq!(vec_compare(out, expected), true);
    }

    #[test]
    fn single_modify() {
        let out = diff("hello, world\n", "hello, test\n");
        let expected = vec![Diff {
            kind: DiffKind::Modify,
            start_line: 1,
            end_line: 1,
        }];
        assert_eq!(vec_compare(out, expected), true);
    }

    #[test]
    fn modify_and_add() {
        let out = diff("hello, world\n", "hello, test\na\nb\n");
        let expected = vec![
            Diff {
                kind: DiffKind::Modify,
                start_line: 1,
                end_line: 1,
            },
            Diff {
                kind: DiffKind::Add,
                start_line: 2,
                end_line: 3,
            },
        ];
        assert_eq!(vec_compare(out, expected), true);
    }

    #[test]
    fn modify_and_delete() {
        let out = diff("hello, world\na\nb\n", "hello, test\n");
        let expected = vec![
            Diff {
                kind: DiffKind::Modify,
                start_line: 1,
                end_line: 1,
            },
            Diff {
                kind: DiffKind::Delete,
                start_line: 2,
                end_line: 2,
            },
        ];
        assert_eq!(vec_compare(out, expected), true);
    }

    #[test]
    fn prefix_add() {
        let out = diff("hello, world\n", "a\nhello, world\n");
        let expected = vec![Diff {
            kind: DiffKind::Add,
            start_line: 1,
            end_line: 1,
        }];
        assert_eq!(vec_compare(out, expected), true);
    }

    #[test]
    fn prefix_delete() {
        let out = diff("a\nhello, world\n", "hello, world\n");
        let expected = vec![Diff {
            kind: DiffKind::Delete,
            start_line: 1,
            end_line: 1,
        }];
        assert_eq!(vec_compare(out, expected), true);
    }

    #[test]
    fn complex() {
        let before = r#"
    #version 330 # to be modified

    in vec4 v_color;
    out vec4 color; # to be removed

    void main() {
        color = v_color;
    };
    to be modified
"#;

        let after = r#"
    #version 331

    in vec4 v_color;

    void main() {
        color = v_color;
        # added this comment
        # and this one
    };
    it was modified
"#;
        let out = diff(before, after);
        let expected = vec![
            Diff {
                kind: DiffKind::Modify,
                start_line: 2,
                end_line: 2,
            },
            Diff {
                kind: DiffKind::Delete,
                start_line: 5,
                end_line: 5,
            },
            Diff {
                kind: DiffKind::Add,
                start_line: 8,
                end_line: 9,
            },
            Diff {
                kind: DiffKind::Modify,
                start_line: 11,
                end_line: 11,
            },
        ];
        assert_eq!(vec_compare(out, expected), true);
    }

    // xxx: generic instead
    fn u8_vec_compare(va: std::vec::Vec<u8>, vb: std::vec::Vec<u8>) -> bool {
        (va.len() == vb.len()) &&  // zip stops at the shortest
         va.iter()
           .zip(vb)
           .all(|(a,b)| *a == b)
    }

    #[test]
    fn wasm_empty() {
        let out = line_diff("hello, world\n2\n3\n4\n", "hello, world\n2\n3\n4\n");
        let expected = vec![];
        assert_eq!(u8_vec_compare(out, expected), true);
    }

    #[test]
    fn wasm_modify_and_delete() {
        let out = line_diff("hello, world\na\nb\n", "hello, test\n");
        let expected = vec![0, 0, 0, 1, 0, 0, 0, 1, 3, 0, 0, 0, 2, 0, 0, 0, 2, 2];
        assert_eq!(u8_vec_compare(out, expected), true);
    }

    #[test]
    fn wasm_single_add() {
        let out = line_diff("", "hello, world\n");
        let expected = vec![0, 0, 0, 1, 0, 0, 0, 1, 1];
        assert_eq!(u8_vec_compare(out, expected), true);
    }
}
