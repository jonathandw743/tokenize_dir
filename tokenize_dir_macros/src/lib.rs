use anyhow::{Result, anyhow};
use proc_macro2::{Literal, Span, TokenTree};
use quote::{format_ident, quote};
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use syn::{LitStr, Token, parse::Parse, punctuated::Punctuated};

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
    files.sort_by(|(file1, _), (file2, _)| file1.path.cmp(&file2.path));
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
