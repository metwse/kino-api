use std::collections::BTreeMap;


/// Trie struct for optimized prefix search queries.
#[derive(Debug, Default)]
pub struct WordTrie {
    next: BTreeMap<u8, WordTrie>,
    word: Option<String>,
}

impl WordTrie {
    /// Creates new WordTrie
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts word to trie. Returns false if word already exists.
    pub fn insert(&mut self, word: String) -> bool{
        let mut current = self;
        for &byte in word.as_bytes() {
            current.next.entry(byte).or_default();

            current = current.next.get_mut(&byte).unwrap() 
        }
        if current.word.is_some() { return false }
        current.word = Some(word);
        true
    }

    /// [`BTreeMap`] prefix search.
    pub fn prefix_search<'a>(&'a self, word: &str, limit: usize) -> Vec<&'a String> {
        let mut current = self;
        for &byte in word.as_bytes() {
            if let Some(current2) = current.next.get(&byte) {
                current = current2
            } else {
                return vec![]
            }
        }

        let mut results = Vec::new();
        let mut candidates = vec![current];

        while !candidates.is_empty() {
            current = candidates.pop().unwrap();

            if let Some(word) = &current.word { results.push(word) }
            if results.len() == limit { break }

            for i in current.next.values() {
                if candidates.len() == limit { break }
                candidates.push(i)
            }
        }

        results
    }

    /// Returns true if [`WordTrie`] has word.
    pub fn has(&self, word: &str) -> bool {
        let mut current = self;
        for &byte in word.as_bytes() {
            if let Some(current2) = current.next.get(&byte) {
                current = current2;
                if current.word.is_some() {
                    return false
                }
            } else {
                break
            }
        }
        false
    }
}
