use std::collections::BTreeMap;

use levenshtein::levenshtein;


/// Levenshtein distance based fuzzy matching tree.
#[derive(Debug, Default)]
pub struct BKTree {
    root: Option<BKTreeNode>
}

#[derive(Debug)]
struct BKTreeNode {
    word: String,
    children: BTreeMap<usize, Vec<BKTreeNode>>,
}

impl BKTree {
    /// Creates new [`BKTree`] object.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts word to tree.
    pub fn insert(&mut self, word: String) {
        if let Some(ref mut root) = self.root {
            root.insert(word);
        } else {
            self.root = Some(BKTreeNode::new(word));
        }
    }

    /// Find closest n words. Not perfect but enough.
    pub fn find<'a>(&'a self, word: &str, limit: usize) -> Vec<&'a String> {
        if let Some(root) = &self.root {
            root.find(word, limit)
        } else { vec![] }
    }
}

impl BKTreeNode {
    fn new(word: String) -> Self {
        Self {
            word,
            children: BTreeMap::new(),
        }
    }

    fn distance(&self, other: &str) -> usize {
        levenshtein(&self.word, other)
    }

    // inserts word to closest child or self
    fn insert(&mut self, word: String) {
        let distance = self.distance(&word);
        let mut closest = None;
        let mut closest_distance = distance;
        let btrees = self.children.values_mut();
        for bknodes in btrees {
            for bknode in bknodes {
                let cur_distance = bknode.distance(&word);
                if cur_distance < closest_distance {
                    closest_distance = cur_distance;
                    closest = Some(bknode)
                }
            }
        }

        if let Some(closest) = closest {
            closest.insert(word)
        } else {
            self.insert_unchecked(word, distance)
        }
    }

    // directly inserts BKTreeNode to child BTreeMap
    fn insert_unchecked(&mut self, word: String, distance: usize) {
        self.children.entry(distance).or_default().push(BKTreeNode::new(word));
    }

    // finds closest words up to limit. optimized by not using recursive functions
    fn find(&self, word: &str, limit: usize) -> Vec<&String> {
        // one of 3 letters might be corrected
        let error_count = word.len() / 3 + (if ((word.len() & 1) + (word.len() & 2)) > 0 { 1 } else { 0 }); 

        let candidates_len = error_count << 12;
        let mut candidates = Vec::with_capacity(candidates_len);
        candidates.push((self, self.distance(word), usize::MAX));
        let mut candidates2 = Vec::with_capacity(candidates_len);

        // nth element of results has n+1 distance
        let mut results = Vec::with_capacity(error_count);
        for _ in 0..error_count { results.push(Vec::new()) }

        let mut total_result = 0;
        let mut tested = 1; // total numbers of levenshtein tests

        'outer: while !candidates.is_empty() {
            std::mem::swap(&mut candidates, &mut candidates2);
            candidates.clear();

            for target in candidates2.iter() {
                let btrees = target.0.children.values();

                for bknodes in btrees {
                    for bknode in bknodes {
                        let cur_distance = bknode.distance(word);
                        tested += 1;
                        if cur_distance <= target.1 && cur_distance < target.2 {
                            if candidates.len() < candidates_len {
                                candidates.push((bknode, cur_distance, target.1));
                            }

                            if cur_distance <= error_count && cur_distance > 0 {
                                results[cur_distance - 1].push(&bknode.word);
                                total_result += 1;
                                if match results[0].len() {
                                    0 => (candidates_len << 2) + candidates_len,
                                    1 => candidates_len << 2,
                                    _ => candidates_len 
                                } < tested || total_result == limit {
                                    break 'outer
                                }
                            }
                        }
                    }
                }

            }
        }

        // concates results into one array
        let mut result2 = Vec::with_capacity(limit);
        for result in results.iter_mut().take(error_count) {
            let mut x = Vec::new();
            std::mem::swap(&mut x, result);
            result2.extend(x);
        }

        result2
    }
}
