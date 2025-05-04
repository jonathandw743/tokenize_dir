pub use tokenize_dir_macros::tokenize_dir;

mod to_iter;

pub use to_iter::ToIter;

pub fn file_indices<'a>(
    mut tokens_associated_files: impl Iterator<Item = impl AsRef<[usize]>>,
) -> Vec<usize> {
    let Some(possible_files) = tokens_associated_files
        .find(|token_associated_files| !token_associated_files.as_ref().is_empty())
    else {
        return Vec::new();
    };
    let mut possible_files = Vec::from(possible_files.as_ref());
    while let Some(token_associated_files) = tokens_associated_files.next() {
        let token_associated_files = token_associated_files.as_ref();
        if possible_files.len() == 1 {
            break;
        }
        if token_associated_files.is_empty() {
            continue;
        }
        if possible_files[possible_files.len() - 1] < token_associated_files[0] {
            continue;
        }
        if possible_files[0] > token_associated_files[token_associated_files.len() - 1] {
            continue;
        }
        let mut i = 0;
        let mut j = 0;
        if possible_files.first().unwrap() > token_associated_files.first().unwrap() {
            let x = possible_files[0];
            let mut b = token_associated_files.len() / 2;
            while b > 0 {
                while j + b < token_associated_files.len() && token_associated_files[j + b] <= x {
                    j += b;
                }
                b /= 2;
            }
        } else
        /* if possible_files.first().unwrap() < token_associated_files.first().unwrap() */
        {
            let x = token_associated_files[0];
            let mut b = possible_files.len() / 2;
            while b > 0 {
                while i + b < possible_files.len() && possible_files[i + b] <= x {
                    i += b;
                }
                b /= 2;
            }
        }
        // let mut new_possible_files = Vec::with_capacity(possible_files.len());
        let mut new_possible_files = Vec::new();
        while i < possible_files.len() && j < token_associated_files.len() {
            if possible_files[i] == token_associated_files[j] {
                new_possible_files.push(possible_files[i]);
                i += 1;
                j += 1;
            } else if possible_files[i] < token_associated_files[j] {
                i += 1;
            } else {
                j += 1;
            }
        }
        if !new_possible_files.is_empty() {
            possible_files = new_possible_files;
        }
    }
    possible_files
}

// #[test]
// fn test() {
//     let a: &[i32] = &[3, 4];
//     let b: &[i32] = &[6, 7];
//     let c: &[&[i32]] = &[a, b];
//     let d: &[&[&[i32]]] = &[c, c];

//     let x = d.iter().flat_map(|x| x.iter());
//     dbg!(x.collect::<Vec<_>>());
// }

// tokenize_dir!("/home/jonathan/git/rust/kenney_input_prompts/kenney_input-prompts_1.4/"; "_");
