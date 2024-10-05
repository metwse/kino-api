use std::{
    collections::BTreeMap, 
    fs,
    path::PathBuf
};

mod word;

use word::{Word, Gloassary};

use super::database::Database;

/// In-memory WordNet database file.
pub struct WordNetDatabase {
    database: Vec<String>,
    index: Vec<BTreeMap<String, Vec<usize>>>,
}

impl WordNetDatabase {
    pub const WORD_TYPES: [&'static str; 4] = ["noun", "verb", "adj", "adv"];

    /// Loads WordNet database.
    pub fn new(location: PathBuf) -> Self {
        let mut database = Vec::with_capacity(4);
        let mut index = Vec::with_capacity(4);

        for file in Self::WORD_TYPES {
            for file_type in ["data", "index"] {
                let mut location = location.clone();
                location.push(format!("dict/{file_type}.{file}"));
                let data = fs::read_to_string(location)
                    .expect("Cannot open database");

                if file_type == "index" {
                    let mut btree = BTreeMap::new();

                    for line in data.lines().into_iter().skip(29) { // skip first 29 license lines
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
            database, index, 
        }
    }

    // Low-level api for fetching a part of word data.
    fn get_by_offset(&self, db: usize, offset: usize) -> Option<Gloassary> {
        // skip first 17 bytes (synset_offset lex_filenum ss_type synset_cnt)
        let lemma_start = offset + 17;
        let mut lemma_end = lemma_start;
        let bytes = self.database[db].as_bytes();
        let synset_cnt_2 = {
            (bytes[offset + 14] - 48) * 10 +
            bytes[offset + 15] - 48
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
        
        Some(Gloassary::new(
            // these from_utf8 functions should not fail. unsafe from_utf8_unchecked might be used
            std::str::from_utf8(&bytes[lemma_start..(lemma_end)]).ok()?,
            std::str::from_utf8(&bytes[glossary_start..=meanings_end]).ok()?.trim(),
            std::str::from_utf8(&bytes[(meanings_end + 1)..=glossary_end]).ok()?.trim(),
        ))
    }
}

impl<'a> Database<'a, Word<'a>> for WordNetDatabase {
    /// Gets word data without copying any &str
    fn get(self: &'a Self, query: String) -> Option<Word<'a>> {
        let mut data: [Vec<Gloassary>; 4];
        data = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut word_exists = false;
        for i in 0..4 {
            if let Some(offsets) = self.index[i].get(&query[..]) {
                word_exists = true;
                data[i] = Vec::with_capacity(offsets.len());
                for &offset in offsets {
                    data[i].push(self.get_by_offset(i, offset)?)
                }
            }
        }

        if word_exists {
            Some(Word {
                lemma: query,
                data
            })
        } else { None }
    }

    fn suggest(&'a self, _query: String) -> Vec<&'a String> {
        todo!()
    }

    fn suggest_search(&'a self, _query: String) -> Vec<&'a String> {
        todo!()
    }
}

