use serde::{
    Serialize,
    ser::{
        Serializer,
        SerializeMap,
        SerializeSeq
    }
};


// Serializable word object which can contain four word types.
pub struct Word<'a> {
    pub lemma: String,
    pub data: [Vec<Gloassary<'a>>; 4],
}

// Single-type word glossary.
pub struct Gloassary<'a> {
    synonyms: Synonyms<'a>, 
    meanings: &'a str,
    examples: Examples<'a>,
}

// Newtype structs for serializing.
struct Synonyms<'a>(&'a str);
struct Examples<'a>(&'a str);

impl<'a> Gloassary<'a> {
    pub fn new(synonyms: &'a str, meanings: &'a str, examles: &'a str) -> Self {
        Self {
            meanings,
            synonyms: Synonyms(synonyms),
            examples: Examples(examles),
        }
    }
}

impl Serialize for Word<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer {
        let mut map = serializer.serialize_map(Some(5))?;

        map.serialize_key("lemma")?;
        map.serialize_value(&self.lemma)?;

        for (i, word_type) in super::WordNetDatabase::WORD_TYPES.iter().enumerate() {
            let data = &self.data[i];
            map.serialize_key(word_type)?;
            map.serialize_value(data)?;
        }

        map.end()
    }
}

impl Serialize for Gloassary<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer {
        let mut map = serializer.serialize_map(Some(3))?;

        map.serialize_key("meanings")?;
        map.serialize_value(&self.meanings)?;
        map.serialize_key("synonyms")?;
        map.serialize_value(&self.synonyms)?;
        map.serialize_key("examples")?;
        map.serialize_value(&self.examples)?;

        map.end()
    }
}

impl Serialize for Synonyms<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer {
        let mut space_count = 1;
        let bytes = self.0.as_bytes();
        for &byte in bytes {
            if byte == b' ' { space_count += 1 }
        }
        space_count >>= 1;

        let mut seq = serializer.serialize_seq(Some(space_count))?;

        let mut word_start = 0;
        let mut skip = false;
        for (i, &byte) in bytes.iter().enumerate() {
            if byte == b' ' { 
                if !skip {
                    seq.serialize_element(
                        std::str::from_utf8(&bytes[word_start..i]).unwrap()
                    )?;
                }
                skip = !skip; 
                word_start = i + 1;
            }
        }

        seq.end()
    }
}

impl Serialize for Examples<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer {
        let mut len = 1;
        let bytes = self.0.as_bytes();
        for &byte in bytes {
            if byte == b';' { len += 1 }
        }
        let mut seq = serializer.serialize_seq(Some(len))?;


        let mut last_start = 0;
        for (i, &byte) in bytes.iter().enumerate().skip(2) {
            if byte == b';' {
                seq.serialize_element(
                    std::str::from_utf8(&bytes[(last_start + 3)..(i - 1)]).unwrap()
                )?;
                last_start = i;
            }
            if i == bytes.len() - 1 { 
                seq.serialize_element(
                    std::str::from_utf8(&bytes[(last_start + 3)..i]).unwrap()
                )?;
            }
        }

        seq.end()
    }
}
