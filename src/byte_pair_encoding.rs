use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::io::{Error, ErrorKind};

pub struct BytePairEncoding {
    pub vocab_size: usize,
    pub tokenizer: HashMap<String, Vec<String>>,
}

impl BytePairEncoding {
    const PUNCTUATION: [char; 6] = [' ', '.', ',', '!', '?', '\n'];
    const START_TOKEN: &str = "<|startoftext|>";
    const END_TOKEN: &str = "<|endoftext|>";

    pub fn from(corpus: String, max_vocab_size: usize) -> Self {
        let vocabulary = Self::build_vocablary(&corpus);
        let mut vocab_size = vocabulary.len() - 2;

        assert!(
            max_vocab_size > vocab_size,
            "vocab_size {} must be greater than the size of the text {}",
            max_vocab_size,
            vocab_size
        );
        if max_vocab_size == vocab_size {
            return BytePairEncoding {
                vocab_size: max_vocab_size,
                tokenizer: HashMap::new(),
            };
        }

        let pre_tokenized = Self::pre_tokenize(&corpus);
        let mut words = Self::text_to_map(&pre_tokenized);

        while max_vocab_size > vocab_size {
            let (pair, freq) = Self::get_most_frequent_pair(&words);
            if freq == 0 {
                break;
            }

            words = Self::merge_by_pair(words, pair);
            vocab_size += 1;
        }

        let tokenizer_mapper =
            words
                .into_keys()
                .fold(HashMap::<String, Vec<String>>::new(), |mut map, word| {
                    map.insert(word.join(""), word);
                    map
                });

        BytePairEncoding {
            vocab_size,
            tokenizer: tokenizer_mapper,
        }
    }

    pub fn tokenize(&self, text: String) -> Result<Vec<String>, Error> {
        let mut tokenized = vec![Self::START_TOKEN.to_string()];

        let pre_tokenized = Self::pre_tokenize(&text);
        for word in pre_tokenized.into_iter() {
            let tokenized_word = self.tokenizer.get(&word).ok_or(Error::new(
                ErrorKind::InvalidInput,
                "Word not found in vocabulary",
            ))?;
            tokenized.extend(tokenized_word.clone());
        }

        tokenized.push(Self::END_TOKEN.to_string());
        Ok(tokenized)
    }

    fn build_vocablary(corpus: &str) -> Vec<String> {
        let alphabet = corpus
            .chars()
            .map(|c| c.to_string())
            .collect::<HashSet<String>>();

        let mut vocabulary = alphabet.into_iter().collect::<Vec<String>>();
        vocabulary.push(Self::START_TOKEN.to_string());
        vocabulary.push(Self::END_TOKEN.to_string());

        vocabulary
    }

    fn pre_tokenize(corpus: &str) -> Vec<String> {
        let mut prepped = vec![];
        let mut word = vec![];

        for c in corpus.chars() {
            if !word.is_empty() && Self::PUNCTUATION.contains(&c) {
                prepped.push(word.join(""));
                word = vec![];
            }

            word.push(c.to_string());
        }
        prepped.push(word.join(""));

        prepped
    }

    fn text_to_map(text: &[String]) -> HashMap<Vec<String>, usize> {
        text.iter()
            .fold(HashMap::<Vec<String>, usize>::new(), |mut words, word| {
                let splitted_word = word.chars().map(|c| c.to_string()).collect::<Vec<String>>();

                *words.entry(splitted_word).or_insert(0) += 1;
                words
            })
    }

    fn get_most_frequent_pair(words: &HashMap<Vec<String>, usize>) -> (Vec<String>, usize) {
        let mut pairs = HashMap::<Vec<String>, usize>::new();
        let (mut most_freq_pair, mut highest_freq) = (vec![], 0);

        for (word, freq) in words.iter() {
            let n = word.len();

            for i in 0..n - 1 {
                let pair = vec![word[i].clone(), word[i + 1].clone()];
                let entry = pairs.entry(pair.clone()).or_insert(0);
                *entry += freq;

                match (*entry).cmp(&highest_freq) {
                    Ordering::Greater => {
                        highest_freq = *entry;
                        most_freq_pair = pair;
                    }
                    Ordering::Equal => {
                        most_freq_pair = most_freq_pair.max(pair);
                    }
                    _ => {}
                }
            }
        }

        (most_freq_pair, highest_freq)
    }

    fn merge_by_pair(
        words: HashMap<Vec<String>, usize>,
        pair: Vec<String>,
    ) -> HashMap<Vec<String>, usize> {
        let mut new_words = HashMap::<Vec<String>, usize>::with_capacity(words.len());
        let pair_str = pair.join("");

        for (word, freq) in words.into_iter() {
            let mut new_word = word.clone();
            let mut i = 0;

            while i < new_word.len() - 1 {
                if new_word[i] == pair[0] && new_word[i + 1] == pair[1] {
                    new_word[i] = pair_str.clone();
                    new_word.remove(i + 1);
                }
                i += 1;
            }

            *new_words.entry(new_word).or_insert(0) += freq;
        }

        new_words
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "a test? yes, a test.";

    fn str_vec_to_string_vec(arr: Vec<&str>) -> Vec<String> {
        arr.into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    }

    #[test]
    fn build_vocablary_returns_unique_characters() {
        let expected = vec![
            " ",
            ",",
            ".",
            "<|endoftext|>",
            "<|startoftext|>",
            "?",
            "a",
            "e",
            "s",
            "t",
            "y",
        ];
        let mut actual = BytePairEncoding::build_vocablary(&TEXT);
        actual.sort();

        assert_eq!(expected, actual);
    }

    #[test]
    fn pre_tokenize_returns_splitted_string() {
        let expected = vec!["a", " test", "?", " yes", ",", " a", " test", "."];
        let actual = BytePairEncoding::pre_tokenize(&TEXT);

        assert_eq!(expected, actual);
    }

    #[test]
    fn text_to_map_returns_map_of_splitted_words_and_their_frequencies() {
        let expected = vec![
            (vec!["a"], 1),
            (vec![" ", "t", "e", "s", "t"], 2),
            (vec!["?"], 1),
            (vec![" ", "y", "e", "s"], 1),
            (vec![","], 1),
            (vec![" ", "a"], 1),
            (vec!["."], 1),
        ]
        .into_iter()
        .map(|(arr, freq)| (str_vec_to_string_vec(arr), freq))
        .collect::<HashMap<Vec<String>, usize>>();

        let pretokenized_text = BytePairEncoding::pre_tokenize(&TEXT);
        let actual = BytePairEncoding::text_to_map(&pretokenized_text);

        assert_eq!(expected, actual);
    }

    #[test]
    fn get_most_frequent_pair_returns_the_most_frequent_pair() {
        let expected = (vec!["e".to_string(), "s".to_string()], 3);

        let pretokenized_text = BytePairEncoding::pre_tokenize(&TEXT);
        let mapped_text = BytePairEncoding::text_to_map(&pretokenized_text);
        let actual = BytePairEncoding::get_most_frequent_pair(&mapped_text);

        assert_eq!(expected, actual);
    }

    #[test]
    fn merge_by_pair_returns_a_new_map_with_merged_words() {
        let pair = vec!["e".to_string(), "s".to_string()];

        let expected = vec![
            (vec!["a"], 1),
            (vec![" ", "t", "es", "t"], 2),
            (vec!["?"], 1),
            (vec![" ", "y", "es"], 1),
            (vec![","], 1),
            (vec![" ", "a"], 1),
            (vec!["."], 1),
        ]
        .into_iter()
        .map(|(arr, freq)| (str_vec_to_string_vec(arr), freq))
        .collect::<HashMap<Vec<String>, usize>>();

        let pretokenized_text = BytePairEncoding::pre_tokenize(&TEXT);
        let mapped_text = BytePairEncoding::text_to_map(&pretokenized_text);
        let actual = BytePairEncoding::merge_by_pair(mapped_text, pair);

        assert_eq!(expected, actual);
    }
}
