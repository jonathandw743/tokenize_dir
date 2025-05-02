use anyhow::{Result, anyhow};
use proc_macro2::{Literal, Span, TokenStream, TokenTree};
use quote::{format_ident, quote};
use regex::Regex;
use std::{
    collections::HashMap,
    path::{Component, Path, PathBuf},
};
use syn::{LitStr, Token, parse::Parse, punctuated::Punctuated};
use walkdir::WalkDir;

// TODO: change entropy so there aren't as many zeros
// TODO: change structure of code output so there are modules that correspond to direcotries
// TODO: and a token appears in a modules if if it appears in all the submodules

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
        .iter()
        .map(|lit| lit.value())
        .collect::<Vec<_>>();
    let dir_paths = dir_paths
        .iter()
        .map(|dir_path| Path::new(dir_path))
        .collect::<Vec<_>>();
    let delimiters = input.delimiters.iter().map(|lit| lit.value());
    let delimiters = Regex::new(
        &delimiters
            .map(|d| regex::escape(&d))
            .collect::<Vec<_>>()
            .join("|"),
    )
    .unwrap();
    tokenize_dir_inner(&dir_paths, &delimiters).unwrap().into()
}

struct File {
    path: PathBuf,

    dir_counts: HashMap<String, usize>,
    stem_word_counts: HashMap<String, usize>,
    ext_counts: HashMap<String, usize>,

    dir_tokens: Vec<(String, usize)>,
    stem_word_tokens: Vec<(String, usize)>,
    ext_tokens: Vec<(String, usize)>,

    entropy: f64,
}

impl File {
    fn from_path(path: PathBuf, delimiters: &Regex) -> Result<Self> {
        let mut dir_counts = HashMap::new();
        let mut dir_count = 0usize;
        if let Some(parent) = path.parent() {
            for component in parent.components() {
                let Component::Normal(dir) = component else {
                    continue;
                };
                let word = dir.to_str().ok_or(anyhow!(""))?;
                *dir_counts.entry(word.to_owned()).or_insert(0usize) += 1;
                dir_count += 1;
            }
        }
        let mut stem = path
            .file_name()
            .ok_or(anyhow!(""))?
            .to_str()
            .ok_or(anyhow!(""))?;
        let mut ext_counts = HashMap::new();
        let mut ext_count = 0usize;
        if let Some((new_stem, exts)) = stem.split_once(".") {
            stem = new_stem;
            for etx in exts.split(".") {
                *ext_counts.entry(etx.to_owned()).or_insert(0usize) += 1;
                ext_count += 1;
            }
        }
        let mut stem_word_counts = HashMap::new();
        let mut stem_word_count = 0usize;
        for word in delimiters.split(stem).filter(|part| !part.is_empty()) {
            *stem_word_counts.entry(word.to_owned()).or_insert(0usize) += 1;
            stem_word_count += 1;
        }

        let mut dir_tokens = Vec::with_capacity(dir_count);
        for (word, &count) in &dir_counts {
            for version in 0..count {
                dir_tokens.push((word.to_owned(), version));
            }
        }
        let mut stem_word_tokens = Vec::with_capacity(stem_word_count);
        for (word, &count) in &stem_word_counts {
            for version in 0..count {
                stem_word_tokens.push((word.to_owned(), version));
            }
        }
        let mut ext_tokens = Vec::with_capacity(ext_count);
        for (word, &count) in &ext_counts {
            for version in 0..count {
                ext_tokens.push((word.to_owned(), version));
            }
        }

        Ok(Self {
            path,

            dir_counts,
            stem_word_counts,
            ext_counts,

            dir_tokens,
            stem_word_tokens,
            ext_tokens,

            entropy: 1.0,
        })
    }
    fn path_lit_str(&self) -> Result<LitStr> {
        let value = self
            .path
            .to_str()
            .ok_or(anyhow!("file path to_str failed"))?;
        Ok(LitStr::new(value, Span::call_site()))
    }
    fn update_entropy(
        &mut self,
        dir_token_counts: &HashMap<(String, usize), usize>,
        stem_token_counts: &HashMap<(String, usize), usize>,
        ext_token_counts: &HashMap<(String, usize), usize>,
        file_count: usize,
    ) {
        self.entropy = 1.0;
        for token in &self.dir_tokens {
            self.entropy *= dir_token_counts[token] as f64 / file_count as f64;
        }
        for token in &self.stem_word_tokens {
            self.entropy *= stem_token_counts[token] as f64 / file_count as f64;
        }
        for token in &self.ext_tokens {
            self.entropy *= ext_token_counts[token] as f64 / file_count as f64;
        }
        // dbg!(self.entropy);
    }
}

fn max_counts(counts: impl Iterator<Item = (String, usize)>) -> Vec<(String, usize)> {
    let mut max_counts = HashMap::new();
    for (word, count) in counts {
        max_counts
            .entry(word)
            .and_modify(|v| {
                if count > *v {
                    *v = count;
                }
            })
            .or_insert(count);
    }
    let mut max_counts = max_counts.into_iter().collect::<Vec<_>>();
    max_counts.sort();
    max_counts
}

fn all_tokens(max_counts: &Vec<(String, usize)>) -> Vec<(&String, Option<usize>)> {
    let mut tokens = Vec::new();
    for (word, max_count) in max_counts {
        if *max_count == 1 {
            tokens.push((word, None));
        } else {
            for i in 0..*max_count {
                tokens.push((word, Some(i)));
            }
        }
    }
    tokens
}

fn tokens_associated_files<'a>(
    max_counts: &Vec<(String, usize)>,
    files: &Vec<File>,
    get_count: fn(&File, &String) -> Option<usize>,
) -> Vec<Vec<usize>> {
    let mut tokens_associated_files = Vec::new();
    for (word, max_count) in max_counts {
        let mut current_tokens_associated_files = vec![Vec::new(); *max_count];
        for (file_index, file) in files.iter().enumerate() {
            let Some(count) = get_count(file, word) else {
                continue;
            };
            for i in 0..count {
                current_tokens_associated_files[i].push(file_index);
            }
        }
        tokens_associated_files.extend_from_slice(&current_tokens_associated_files);
    }
    tokens_associated_files
}

fn constant(token: &(&String, Option<usize>), token_associated_files: &Vec<usize>) -> TokenStream {
    #[cfg(debug_assertions)]
    assert!(token_associated_files.is_sorted());

    let constant_name = match token.1 {
        None => format_ident!(
            "_{}",
            token
                .0
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '_' })
                .collect::<String>()
        ),
        Some(i) => format_ident!(
            "_{}_{}",
            token
                .0
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '_' })
                .collect::<String>(),
            i
        ),
    };
    let array_elements = token_associated_files
        .iter()
        .map(|file_index| TokenTree::Literal(Literal::usize_unsuffixed(*file_index)));

    quote! {
        pub const #constant_name: &[usize] = &[#(#array_elements,)*];
    }
}

fn tokenize_dir_inner<P: AsRef<Path>>(
    dir_paths: &Vec<P>,
    delimiters: &Regex,
) -> Result<proc_macro2::TokenStream> {
    let mut files = Vec::new();
    let mut dir_token_counts = HashMap::new();
    let mut stem_token_counts = HashMap::new();
    let mut ext_token_counts = HashMap::new();
    for dir_path in dir_paths {
        for dir_entry in WalkDir::new(dir_path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let path = dir_entry.path();
            if path.is_file() {
                let file = File::from_path(path.to_owned(), delimiters)?;
                for token in &file.dir_tokens {
                    dir_token_counts
                        .entry(token.to_owned())
                        .and_modify(|count| *count += 1)
                        .or_insert(0usize);
                }
                for token in &file.stem_word_tokens {
                    stem_token_counts
                        .entry(token.to_owned())
                        .and_modify(|count| *count += 1)
                        .or_insert(0usize);
                }
                for token in &file.ext_tokens {
                    ext_token_counts
                        .entry(token.to_owned())
                        .and_modify(|count| *count += 1)
                        .or_insert(0usize);
                }
                files.push(file);
            }
        }
    }
    let file_count = files.len();
    for file in &mut files {
        file.update_entropy(
            &dir_token_counts,
            &stem_token_counts,
            &ext_token_counts,
            file_count,
        );
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    files.sort_by(|a, b| {
        a.entropy
            .partial_cmp(&b.entropy)
            .expect("float ordering failed")
            .reverse()
    });

    let mut file_paths_elements = Vec::new();
    for file in &files {
        file_paths_elements.push(file.path_lit_str()?);
    }

    let dir_max_counts = max_counts(files.iter().flat_map(|file| file.dir_counts.clone()));
    let stem_word_max_counts =
        max_counts(files.iter().flat_map(|file| file.stem_word_counts.clone()));
    let ext_max_counts = max_counts(files.iter().flat_map(|file| file.ext_counts.clone()));

    let all_dir_tokens = all_tokens(&dir_max_counts);
    let all_stem_word_tokens = all_tokens(&stem_word_max_counts);
    let all_ext_tokens = all_tokens(&ext_max_counts);

    let dir_tokens_associated_files =
        tokens_associated_files(&dir_max_counts, &files, |file, word| {
            file.dir_counts.get(word).copied()
        });
    let stem_word_tokens_associated_files =
        tokens_associated_files(&stem_word_max_counts, &files, |file, word| {
            file.stem_word_counts.get(word).copied()
        });
    let ext_tokens_associated_files =
        tokens_associated_files(&ext_max_counts, &files, |file, word| {
            file.ext_counts.get(word).copied()
        });

    let dir_constants = all_dir_tokens
        .iter()
        .zip(&dir_tokens_associated_files)
        .map(|(token, token_associated_files)| constant(token, token_associated_files));
    let stem_word_constants = all_stem_word_tokens
        .iter()
        .zip(&stem_word_tokens_associated_files)
        .map(|(token, token_associated_files)| constant(token, token_associated_files));
    let ext_constants = all_ext_tokens
        .iter()
        .zip(&ext_tokens_associated_files)
        .map(|(token, token_associated_files)| constant(token, token_associated_files));

    Ok(quote! {
        pub const FILE_PATHS: &[&str] = &[
            #(#file_paths_elements,)*
        ];
        pub mod dir {
            #(#dir_constants)*
        }
        pub mod stem_word {
            #(#stem_word_constants)*
        }
        pub mod ext {
            #(#ext_constants)*
        }
    })
}
