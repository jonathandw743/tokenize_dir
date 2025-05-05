use anyhow::{Result, anyhow};
use proc_macro2::{Literal, Span, TokenTree};
use quote::{format_ident, quote};
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    default, fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use syn::{LitStr, Token, parse::Parse, punctuated::Punctuated};

// TODO: order files based on entropy
// TODO: change entropy so there aren't as many zeroes

struct Input {
    dir_paths: Punctuated<LitStr, Token![,]>,
    _comma: Token![;],
    delimiters: Punctuated<LitStr, Token![,]>,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            dir_paths: Punctuated::parse_separated_nonempty(input)?,
            _comma: input.parse()?,
            delimiters: Punctuated::parse_terminated(input)?,
        })
    }
}

#[proc_macro]
pub fn tokenize_dir(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as Input);
    let dir_paths = input
        .dir_paths
        .into_iter()
        .map(|lit| lit.value())
        .collect::<Vec<_>>();
    let dir_paths = dir_paths
        .into_iter()
        .map(|dir_path| PathBuf::from_str(&dir_path).unwrap());
    let delimiters = input.delimiters.iter().map(|lit| lit.value());
    let delimiters = Regex::new(
        &delimiters
            .map(|d| regex::escape(&d))
            .collect::<Vec<_>>()
            .join("|"),
    )
    .unwrap();
    tokenize_dir_inner(&dir_paths.collect::<Vec<_>>(), &delimiters)
        .unwrap()
        .into()
}

// struct File {
//     path: PathBuf,

//     dir_counts: HashMap<String, usize>,
//     stem_word_counts: HashMap<String, usize>,
//     ext_counts: HashMap<String, usize>,

//     dir_tokens: Vec<(String, usize)>,
//     stem_word_tokens: Vec<(String, usize)>,
//     ext_tokens: Vec<(String, usize)>,

//     entropy: f64,
// }

// impl File {
//     fn from_path(path: PathBuf, delimiters: &Regex) -> Result<Self> {
//         let mut dir_counts = HashMap::new();
//         let mut dir_count = 0usize;
//         if let Some(parent) = path.parent() {
//             for component in parent.components() {
//                 let Component::Normal(dir) = component else {
//                     continue;
//                 };
//                 let word = dir.to_str().ok_or(anyhow!(""))?;
//                 *dir_counts.entry(word.to_owned()).or_insert(0usize) += 1;
//                 dir_count += 1;
//             }
//         }
//         let mut stem = path
//             .file_name()
//             .ok_or(anyhow!(""))?
//             .to_str()
//             .ok_or(anyhow!(""))?;
//         let mut ext_counts = HashMap::new();
//         let mut ext_count = 0usize;
//         if let Some((new_stem, exts)) = stem.split_once(".") {
//             stem = new_stem;
//             for etx in exts.split(".") {
//                 *ext_counts.entry(etx.to_owned()).or_insert(0usize) += 1;
//                 ext_count += 1;
//             }
//         }
//         let mut stem_word_counts = HashMap::new();
//         let mut stem_word_count = 0usize;
//         for word in delimiters.split(stem).filter(|part| !part.is_empty()) {
//             *stem_word_counts.entry(word.to_owned()).or_insert(0usize) += 1;
//             stem_word_count += 1;
//         }

//         let mut dir_tokens = Vec::with_capacity(dir_count);
//         for (word, &count) in &dir_counts {
//             for version in 0..count {
//                 dir_tokens.push((word.to_owned(), version));
//             }
//         }
//         let mut stem_word_tokens = Vec::with_capacity(stem_word_count);
//         for (word, &count) in &stem_word_counts {
//             for version in 0..count {
//                 stem_word_tokens.push((word.to_owned(), version));
//             }
//         }
//         let mut ext_tokens = Vec::with_capacity(ext_count);
//         for (word, &count) in &ext_counts {
//             for version in 0..count {
//                 ext_tokens.push((word.to_owned(), version));
//             }
//         }

//         Ok(Self {
//             path,

//             dir_counts,
//             stem_word_counts,
//             ext_counts,

//             dir_tokens,
//             stem_word_tokens,
//             ext_tokens,

//             entropy: 1.0,
//         })
//     }
//     fn path_lit_str(&self) -> Result<LitStr> {
//         let value = self
//             .path
//             .to_str()
//             .ok_or(anyhow!("file path to_str failed"))?;
//         Ok(LitStr::new(value, Span::call_site()))
//     }
//     fn update_entropy(
//         &mut self,
//         dir_token_counts: &HashMap<(String, usize), usize>,
//         stem_token_counts: &HashMap<(String, usize), usize>,
//         ext_token_counts: &HashMap<(String, usize), usize>,
//         file_count: usize,
//     ) {
//         self.entropy = 1.0;
//         for token in &self.dir_tokens {
//             self.entropy *= dir_token_counts[token] as f64 / file_count as f64;
//         }
//         for token in &self.stem_word_tokens {
//             self.entropy *= stem_token_counts[token] as f64 / file_count as f64;
//         }
//         for token in &self.ext_tokens {
//             self.entropy *= ext_token_counts[token] as f64 / file_count as f64;
//         }
//         // dbg!(self.entropy);
//     }
// }

// fn max_counts(counts: impl Iterator<Item = (String, usize)>) -> Vec<(String, usize)> {
//     let mut max_counts = HashMap::new();
//     for (word, count) in counts {
//         max_counts
//             .entry(word)
//             .and_modify(|v| {
//                 if count > *v {
//                     *v = count;
//                 }
//             })
//             .or_insert(count);
//     }
//     let mut max_counts = max_counts.into_iter().collect::<Vec<_>>();
//     max_counts.sort();
//     max_counts
// }

// fn all_tokens(max_counts: &Vec<(String, usize)>) -> Vec<(&String, Option<usize>)> {
//     let mut tokens = Vec::new();
//     for (word, max_count) in max_counts {
//         if *max_count == 1 {
//             tokens.push((word, None));
//         } else {
//             for i in 0..*max_count {
//                 tokens.push((word, Some(i)));
//             }
//         }
//     }
//     tokens
// }

// fn tokens_associated_files<'a>(
//     max_counts: &Vec<(String, usize)>,
//     files: &Vec<File>,
//     get_count: fn(&File, &String) -> Option<usize>,
// ) -> Vec<Vec<usize>> {
//     let mut tokens_associated_files = Vec::new();
//     for (word, max_count) in max_counts {
//         let mut current_tokens_associated_files = vec![Vec::new(); *max_count];
//         for (file_index, file) in files.iter().enumerate() {
//             let Some(count) = get_count(file, word) else {
//                 continue;
//             };
//             for i in 0..count {
//                 current_tokens_associated_files[i].push(file_index);
//             }
//         }
//         tokens_associated_files.extend_from_slice(&current_tokens_associated_files);
//     }
//     tokens_associated_files
// }

// fn constant(token: &(&String, Option<usize>), token_associated_files: &Vec<usize>) -> TokenStream {
//     #[cfg(debug_assertions)]
//     assert!(token_associated_files.is_sorted());

//     let constant_name = match token.1 {
//         None => format_ident!(
//             "_{}",
//             token
//                 .0
//                 .chars()
//                 .map(|c| if c.is_alphanumeric() { c } else { '_' })
//                 .collect::<String>()
//         ),
//         Some(i) => format_ident!(
//             "_{}_{}",
//             token
//                 .0
//                 .chars()
//                 .map(|c| if c.is_alphanumeric() { c } else { '_' })
//                 .collect::<String>(),
//             i
//         ),
//     };
//     let array_elements = token_associated_files
//         .iter()
//         .map(|file_index| TokenTree::Literal(Literal::usize_unsuffixed(*file_index)));

//     quote! {
//         pub const #constant_name: &[usize] = &[#(#array_elements,)*];
//     }
// }

#[derive(Debug, Default, Clone)]
struct File {
    // should be unique
    path: PathBuf,
    stem_word_tokens: HashSet<(String, usize)>,
    ext_tokens: HashSet<(String, usize)>,
    num_files_in_dir: usize,
}

impl File {
    fn negative_log_likelihood(
        &self,
        all_stem_word_tokens: &HashMap<(String, usize), HashSet<PathBuf>>,
        all_ext_tokens: &HashMap<(String, usize), HashSet<PathBuf>>,
        num_files: usize,
    ) -> usize {
        let mut l = 0;
        for stem_word_token in &self.stem_word_tokens {
            l += num_files;
            l -= all_stem_word_tokens[stem_word_token].len();
        }
        for ext_token in &self.ext_tokens {
            l += num_files;
            l -= all_ext_tokens[ext_token].len();
        }
        l += num_files;
        l -= self.num_files_in_dir;
        l
    }
}

#[derive(Debug)]
struct Directory {
    files: Vec<File>,
    name: String,
    sub_dirs: Vec<Directory>,
    stem_word_tokens: HashMap<(String, usize), HashSet<PathBuf>>,
    ext_tokens: HashMap<(String, usize), HashSet<PathBuf>>,
}

fn tokenize_dir_inner_inner<P: AsRef<Path>>(path: P, delimiters: &Regex) -> Result<Directory> {
    let mut files = Vec::new();
    let mut children = Vec::new();
    let dir = path
        .as_ref()
        .file_name()
        .ok_or(anyhow!(""))?
        .to_str()
        .ok_or(anyhow!("to_str failed"))?
        .to_owned();
    let mut num_files_in_dir = 0;
    for dir_entry in fs::read_dir(path).unwrap() {
        let path = dir_entry?.path();
        if path.is_file() {
            files.push(File {
                path,
                ..Default::default()
            });
            num_files_in_dir += 1;
        } else if path.is_dir() {
            let foo = tokenize_dir_inner_inner(path, delimiters)?;
            children.push(foo);
        }
    }
    for file in &mut files {
        file.num_files_in_dir = num_files_in_dir;
    }
    let mut file_names = Vec::new();
    for file in &files {
        file_names.push(
            file.path
                .file_name()
                .ok_or(anyhow!("file_name failed"))?
                .to_str()
                .ok_or(anyhow!("to_str failed"))?
                .to_owned(),
        );
    }
    let mut stem_word_tokens: HashMap<(String, usize), HashSet<PathBuf>> = HashMap::new();
    let mut ext_tokens: HashMap<(String, usize), HashSet<PathBuf>> = HashMap::new();
    for (file, file_name) in files.iter_mut().zip(file_names.into_iter()) {
        let mut stem = file_name.clone();
        let mut ext_counts = HashMap::new();
        if let Some((new_stem, exts)) = file_name.split_once(".") {
            stem = new_stem.to_owned();
            for etx in exts.split(".") {
                *ext_counts.entry(etx.to_owned()).or_insert(0usize) += 1;
            }
        }
        let mut stem_word_counts = HashMap::new();
        for word in delimiters.split(&stem).filter(|part| !part.is_empty()) {
            *stem_word_counts.entry(word.to_owned()).or_insert(0usize) += 1;
        }
        for (word, &count) in &stem_word_counts {
            for version in 0..count {
                stem_word_tokens
                    .entry((word.to_owned(), version))
                    .or_default()
                    .insert(file.path.clone());
                file.stem_word_tokens.insert((word.to_owned(), version));
            }
        }
        for (word, &count) in &ext_counts {
            for version in 0..count {
                ext_tokens
                    .entry((word.to_owned(), version))
                    .or_default()
                    .insert(file.path.clone());
                file.ext_tokens.insert((word.to_owned(), version));
            }
        }
    }
    if !children.is_empty() {
        // let mut stem_word_tokens_multiple: HashMap<(String, usize), HashSet<PathBuf>> =
        //     HashMap::new();
        // for i in 0..children.len() {
        //     for j in (i + 1)..children.len() {
        //         for (stem_word_token_0, paths_0) in &children[i].stem_word_tokens {
        //             for (stem_word_token_1, paths_1) in &children[j].stem_word_tokens {
        //                 if stem_word_token_0 == stem_word_token_1 {
        //                     let paths = stem_word_tokens_multiple
        //                         .entry(stem_word_token_0.clone())
        //                         .or_default();
        //                     for path in paths_0 {
        //                         paths.insert(path.clone());
        //                     }
        //                     for path in paths_1 {
        //                         paths.insert(path.clone());
        //                     }
        //                 }
        //             }
        //         }
        //     }
        // }
        for child in &children {
            for (stem_word_token, paths) in &child.stem_word_tokens {
                // stem_word_tokens_multiple {
                // for child in &mut children {
                //     child.stem_word_tokens.remove(&stem_word_token);
                // }
                let full_paths = stem_word_tokens.entry(stem_word_token.clone()).or_default();
                for path in paths {
                    full_paths.insert(path.clone());
                }
            }
        }
        // let mut ext_tokens_multiple: HashMap<(String, usize), HashSet<PathBuf>> = HashMap::new();
        // for i in 0..children.len() {
        //     for j in (i + 1)..children.len() {
        //         for (ext_token_0, paths_0) in &children[i].ext_tokens {
        //             for (ext_token_1, paths_1) in &children[j].ext_tokens {
        //                 if ext_token_0 == ext_token_1 {
        //                     let paths = ext_tokens_multiple.entry(ext_token_0.clone()).or_default();
        //                     for path in paths_0 {
        //                         paths.insert(path.clone());
        //                     }
        //                     for path in paths_1 {
        //                         paths.insert(path.clone());
        //                     }
        //                 }
        //             }
        //         }
        //     }
        // }
        for child in &children {
            for (ext_token, paths) in &child.ext_tokens {
                // ext_tokens_multiple {
                // for child in &mut children {
                //     child.ext_tokens.remove(&ext_token);
                // }
                let full_paths = ext_tokens.entry(ext_token.clone()).or_default();
                for path in paths {
                    full_paths.insert(path.clone());
                }
            }
        }
    }
    for child in &mut children {
        for file in &child.files {
            files.push(file.clone());
        }
    }
    Ok(Directory {
        files,
        name: dir,
        sub_dirs: children,
        stem_word_tokens,
        ext_tokens,
    })
}

fn create_const_arrays(
    tokens: &HashMap<(String, usize), HashSet<PathBuf>>,
    file_to_index: &HashMap<PathBuf, usize>,
) -> impl Iterator<Item = proc_macro2::TokenStream> {
    let mut max_is = HashMap::new();
    for (word, i) in tokens.keys() {
        max_is
            .entry(word.clone())
            .and_modify(|x: &mut usize| *x = (*x).max(*i))
            .or_insert(*i);
    }
    tokens.iter().map(move |((word, i), files)| {
        let word = if max_is[word] == 0 {
            let word = word
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '_' })
                .collect::<String>();
            format_ident!("_{}", word)
        } else {
            let word = word
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '_' })
                .collect::<String>();
            format_ident!("_{}_{}", word, i)
        };
        let mut file_indices = files
            .iter()
            .map(|file| file_to_index[file])
            .collect::<Vec<_>>();
        file_indices.sort();
        let file_indices = file_indices
            .into_iter()
            .map(|file_index| TokenTree::Literal(Literal::usize_unsuffixed(file_index)));
        quote! {
            pub const #word: &[usize] = &[ #(#file_indices,)* ];
        }
    })
}

fn create_ts(foo: &Directory, file_to_index: &HashMap<PathBuf, usize>) -> proc_macro2::TokenStream {
    let dir = foo
        .name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>();
    let dir = format_ident!("_{}", dir);
    let mut file_indices = foo
        .files
        .iter()
        .map(|file| file_to_index[&file.path])
        .collect::<Vec<_>>();
    file_indices.sort();
    let file_indices = file_indices
        .into_iter()
        .map(|file_index| TokenTree::Literal(Literal::usize_unsuffixed(file_index)));
    let stem_word_tokens = create_const_arrays(&foo.stem_word_tokens, file_to_index);
    let ext_tokens = create_const_arrays(&foo.ext_tokens, file_to_index);
    let children = foo
        .sub_dirs
        .iter()
        .map(|child| create_ts(child, file_to_index));
    quote! {
        pub mod #dir {
            pub const DIR: &[usize] = &[ #(#file_indices,)* ];
            pub mod stem_words {
                #(#stem_word_tokens)*
            }
            pub mod exts {
                #(#ext_tokens)*
            }
            #(#children)*
        }
    }
}

fn tokenize_dir_inner<P: AsRef<Path>>(
    dir_paths: &Vec<P>,
    delimiters: &Regex,
) -> Result<proc_macro2::TokenStream> {
    let mut files = Vec::new();
    let mut directories = Vec::new();
    for (i, dir_path) in dir_paths.iter().enumerate() {
        let directory = tokenize_dir_inner_inner(dir_path, delimiters)?;
        for file in &directory.files {
            files.push((file.clone(), i));
        }
        directories.push(directory);
    }
    let num_files = files.len();
    files.sort_by_key(|(file, dir_index)| {
        file.negative_log_likelihood(
            &directories[*dir_index].stem_word_tokens,
            &directories[*dir_index].ext_tokens,
            num_files,
        )
    });
    let mut files_to_index = HashMap::new();
    for (index, (file, _dir_index)) in files.iter().enumerate() {
        files_to_index.insert(file.path.clone(), index);
    }
    let mut file_lits = Vec::new();
    for (file, _dir_index) in files {
        file_lits.push(LitStr::new(
            file.path.to_str().ok_or(anyhow!(""))?,
            Span::call_site(),
        ));
    }
    let foos = directories
        .iter()
        .map(|foo| create_ts(foo, &files_to_index));
    Ok(quote! {
        pub const FILE_PATHS: &[&str] = &[ #(#file_lits,)* ];
        #(#foos)*
    })
}

// fn tokenize_dir_inner<P: AsRef<Path>>(
//     dir_paths: &Vec<P>,
//     delimiters: &Regex,
// ) -> Result<proc_macro2::TokenStream> {
//     let mut files = Vec::new();
//     let mut dir_token_counts = HashMap::new();
//     let mut stem_token_counts = HashMap::new();
//     let mut ext_token_counts = HashMap::new();
//     for dir_path in dir_paths {
//         for dir_entry in WalkDir::new(dir_path)
//             .into_iter()
//             .filter_map(Result::ok)
//             .filter(|e| e.file_type().is_file())
//         {
//             let path = dir_entry.path();
//             if path.is_file() {
//                 let file = File::from_path(path.to_owned(), delimiters)?;
//                 for token in &file.dir_tokens {
//                     dir_token_counts
//                         .entry(token.to_owned())
//                         .and_modify(|count| *count += 1)
//                         .or_insert(0usize);
//                 }
//                 for token in &file.stem_word_tokens {
//                     stem_token_counts
//                         .entry(token.to_owned())
//                         .and_modify(|count| *count += 1)
//                         .or_insert(0usize);
//                 }
//                 for token in &file.ext_tokens {
//                     ext_token_counts
//                         .entry(token.to_owned())
//                         .and_modify(|count| *count += 1)
//                         .or_insert(0usize);
//                 }
//                 files.push(file);
//             }
//         }
//     }
//     let file_count = files.len();
//     for file in &mut files {
//         file.update_entropy(
//             &dir_token_counts,
//             &stem_token_counts,
//             &ext_token_counts,
//             file_count,
//         );
//     }
//     files.sort_by(|a, b| a.path.cmp(&b.path));
//     files.sort_by(|a, b| {
//         a.entropy
//             .partial_cmp(&b.entropy)
//             .expect("float ordering failed")
//             .reverse()
//     });

//     let mut file_paths_elements = Vec::new();
//     for file in &files {
//         file_paths_elements.push(file.path_lit_str()?);
//     }

//     let dir_max_counts = max_counts(files.iter().flat_map(|file| file.dir_counts.clone()));
//     let stem_word_max_counts =
//         max_counts(files.iter().flat_map(|file| file.stem_word_counts.clone()));
//     let ext_max_counts = max_counts(files.iter().flat_map(|file| file.ext_counts.clone()));

//     let all_dir_tokens = all_tokens(&dir_max_counts);
//     let all_stem_word_tokens = all_tokens(&stem_word_max_counts);
//     let all_ext_tokens = all_tokens(&ext_max_counts);

//     let dir_tokens_associated_files =
//         tokens_associated_files(&dir_max_counts, &files, |file, word| {
//             file.dir_counts.get(word).copied()
//         });
//     let stem_word_tokens_associated_files =
//         tokens_associated_files(&stem_word_max_counts, &files, |file, word| {
//             file.stem_word_counts.get(word).copied()
//         });
//     let ext_tokens_associated_files =
//         tokens_associated_files(&ext_max_counts, &files, |file, word| {
//             file.ext_counts.get(word).copied()
//         });

//     let dir_constants = all_dir_tokens
//         .iter()
//         .zip(&dir_tokens_associated_files)
//         .map(|(token, token_associated_files)| constant(token, token_associated_files));
//     let stem_word_constants = all_stem_word_tokens
//         .iter()
//         .zip(&stem_word_tokens_associated_files)
//         .map(|(token, token_associated_files)| constant(token, token_associated_files));
//     let ext_constants = all_ext_tokens
//         .iter()
//         .zip(&ext_tokens_associated_files)
//         .map(|(token, token_associated_files)| constant(token, token_associated_files));

//     Ok(quote! {
//         pub const FILE_PATHS: &[&str] = &[
//             #(#file_paths_elements,)*
//         ];
//         pub mod dir {
//             #(#dir_constants)*
//         }
//         pub mod stem_word {
//             #(#stem_word_constants)*
//         }
//         pub mod ext {
//             #(#ext_constants)*
//         }
//     })
// }
