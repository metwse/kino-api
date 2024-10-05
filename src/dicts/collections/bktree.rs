use std::collections::BTreeMap;
use levenshtein::levenshtein;

#[derive(Debug)]
pub struct BKTree {
    root: Option<BKTreeNode>
}

#[derive(Debug)]
struct BKTreeNode {
    word: String,
    value: BTreeMap<usize, Vec<BKTreeNode>>,
}

impl BKTree {
    pub fn new() -> Self {
        Self {
            root: None
        }
    }

    pub fn insert(&mut self, word: String) {
        if let Some(ref mut root) = self.root {
            root.insert(word);
        } else {
            self.root = Some(BKTreeNode::new(word));
        }
    }

    pub fn find<'a>(&'a self, word: String, limit: usize) -> Vec<&'a String> {
        if let Some(root) = &self.root {
            root.find(&word, limit)
        } else { vec![] }
    }
}

impl BKTreeNode {
    fn new(word: String) -> Self {
        Self {
            word,
            value: BTreeMap::new(),
        }
    }

    fn distance(&self, other: &String) -> usize {
        levenshtein(&self.word, other)
    }

    fn insert(&mut self, word: String) {
        let distance = self.distance(&word);
        let mut closest = None;
        let mut closest_distance = distance;
        let btrees = self.value.values_mut();
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

    fn insert_unchecked(&mut self, word: String, distance: usize) {
        if let None = self.value.get(&distance) {
            self.value.insert(distance, Vec::new());
        }
        self.value.get_mut(&distance).unwrap().push(BKTreeNode::new(word));
    }

    fn find(&self, word: &String, limit: usize) -> Vec<&String> {
        if word.len() > 20 { return vec![] }

        let mut targets = vec![(self, self.distance(word), std::usize::MAX)];
        let error_count = word.len() / 3 + 1;
        let mut results = Vec::with_capacity(error_count);
        for _ in 0..error_count {
            results.push(Vec::new())
        }
        let mut total_result = 0;
        let mut tested = 0;
        'outer: while targets.len() > 0 {
            let mut targets2 = Vec::new();
            std::mem::swap(&mut targets, &mut targets2);
            for target in targets2.iter() {
                tested += 1;
                let btrees = target.0.value.values();

                'inner: for bknodes in btrees {
                    for bknode in bknodes {
                        if targets.len() >= 30000 { break 'inner }
                        let cur_distance = bknode.distance(&word);
                        if cur_distance <= target.1 && cur_distance < target.2 {
                            targets.push((bknode, cur_distance, target.1));
                            if cur_distance < error_count {
                                results[cur_distance].push(&bknode.word);
                                total_result += 1;
                                if (total_result > 0 && tested > error_count * 5000) || (total_result == 0 && tested > error_count * 10000) || (total_result >= limit) {
                                    break 'outer
                                }
                            }
                        }
                    }
                }

            }
        }

        let mut result2 = Vec::new();
        for i in 0..error_count {
            let mut x = Vec::new();
            std::mem::swap(&mut x, &mut results[i]);
            result2.extend(x);
        }

        result2
    }
}
