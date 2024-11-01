mod word;

use word::{Word, Gloassary};

use super::{
    database::Database,
    collections::{
        WordTrie,
        BKTree,
    },
};

use std::{
    collections::BTreeMap, 
    fs,
    path::PathBuf,
};


/// In-memory WordNet database file.
pub struct WordNetDatabase {
    database: Vec<String>,
    word_trie: WordTrie,
    /// BKTree is not initialized to improve performance if debug mode enabled.
    bktree: BKTree,
    index: Vec<BTreeMap<String, Vec<usize>>>,
}

impl WordNetDatabase {
    pub const WORD_TYPES: [&'static str; 4] = ["noun", "verb", "adj", "adv"];

    /// Loads WordNet database.
    pub fn new(location: PathBuf) -> Self {
        let mut database = Vec::with_capacity(4);
        let mut word_trie = WordTrie::new();
        let mut bktree = BKTree::new();
        let mut index = Vec::with_capacity(4);
        tracing::info!("BKTree disabled in debug mode");

        for file in Self::WORD_TYPES {
            for file_type in ["data", "index"] {
                let mut location = location.clone();
                location.push(format!("{file_type}.{file}"));
                let data = fs::read_to_string(location)
                    .expect("Cannot open database");

                if file_type == "index" {
                    let mut btree = BTreeMap::new();

                    for line in data.lines().skip(29) { // skip first 29 license lines
                        // index files might end with 2 to 4 spaces. because of that trim()
                        // required
                        let line = line.trim().split(" ").collect::<Vec<_>>();
                        let lemma = line[0].to_owned();
                        let synset_cnt: usize = line[2].parse().unwrap();
                        let mut synset_offsets: Vec<usize> = Vec::with_capacity(synset_cnt);

                        // synsets at end of line
                        for i in 0..synset_cnt {
                            synset_offsets.push(
                                line[line.len() - 1 - i].parse().unwrap()
                            )
                        }

                        // disables bktree in debug mode because of lack of performance
                        if !cfg!(debug_assertions) && word_trie.insert(lemma.clone()) {
                            bktree.insert(lemma.clone());
                        }
                        btree.insert(lemma, synset_offsets);
                    }

                    index.push(btree)
                } else {
                    // data.noun, data.verb, data.adj, data.adv as String
                    database.push(data)
                }
            }
        }

        Self {
            database,
            word_trie,
            bktree,
            index, 
        }
    }

    // Low-level API for fetching a part of word data.
    fn get_by_offset(&self, db: usize, offset: usize) -> Option<Gloassary> {
        // skip first 17 bytes (synset_offset lex_filenum ss_type synset_cnt)
        let lemma_start = offset + 17;
        let mut lemma_end = lemma_start;
        let bytes = self.database[db].as_bytes();
        let base16_char_to_num = |c: u8| if c < 58 { c - 48 } else { c - 87 };
        let synset_cnt_2 = {
            base16_char_to_num(bytes[offset + 14]) * 16 +
            base16_char_to_num(bytes[offset + 15])
        } * 2; // for each lemma, two spaces must be skipped

        let mut collected_lemmas = 0;
        while collected_lemmas < synset_cnt_2 {
            if bytes[lemma_end] == b' ' { collected_lemmas += 1; }
            lemma_end += 1
        }
        

        let mut glossary_start = lemma_end;
        while bytes[glossary_start - 1] != b'|' { glossary_start += 1 }

        let mut glossary_end = glossary_start;
        let mut meanings_end = glossary_start;
        while bytes[glossary_end + 1] != b'\n' { 
            if bytes[glossary_end] == b';' && bytes[glossary_end + 2] == b'"' && meanings_end == glossary_start { 
                meanings_end = glossary_end - 1;
            }
            glossary_end += 1 
        } 
        if meanings_end == glossary_start {
            meanings_end = glossary_end - 1; 
        }

        // these from_utf8 functions should not fail. unsafe from_utf8_unchecked might be used
        let lemma = std::str::from_utf8(&bytes[lemma_start..(lemma_end)]).ok()?;
        let glossary = std::str::from_utf8(&bytes[glossary_start..=meanings_end]).ok()?.trim();
        let examples = std::str::from_utf8(&bytes[(meanings_end + 1)..=glossary_end]).ok()?.trim();
        Some(Gloassary::new(
            lemma,
            glossary,
            examples
        ))
    }
}

impl<'a> Database<'a, Word<'a>> for WordNetDatabase {
    /// Gets word data without copying any &str
    fn get(&'a self, query: &str) -> Option<Word<'a>> {
        let mut data: [Vec<Gloassary>; 4];
        data = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut word_exists = false;
        for (i, data) in data.iter_mut().enumerate() {
            if let Some(offsets) = self.index[i].get(query) {
                word_exists = true;
                *data = Vec::with_capacity(offsets.len());
                for &offset in offsets {
                    data.push(self.get_by_offset(i, offset)?)
                }
            }
        }

        if word_exists {
            Some(Word {
                lemma: query.to_string(),
                data
            })
        } else { None }
    }

    fn suggest(&'a self, query: &str) -> Vec<&'a String> {
        self.bktree.find(query, 8)
    }

    fn suggest_search(&'a self, query: &str) -> Vec<&'a String> {
        self.word_trie.prefix_search(query, 8)
    }
}
