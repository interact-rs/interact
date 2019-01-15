//! The `Expect` struct manages the tree of expectations during parsing.
//!
//! While parsing an expression for `Token`s, the parser may have expectations at each point in the
//! way of what tokens should appear. In case these tokens do not show up, we can remember what
//! tokens could have appeared and then backtrack the parsing. This crates a tree that can be later
//! flattened in order to present a list of options.
//!
//! This allows to implement auto-completion and hints so that users can type the minimal amount of
//! characters. That is especially relevant in the case where there's only one token that can
//! follow.

#[derive(Debug, Eq, PartialEq, Clone)]
struct Expect<T> {
    item: T,
    next: Vec<Expect<T>>,
}

impl<T> Expect<T>
where
    T: Eq + PartialEq + Clone,
{
    fn into_flatten(self) -> Vec<Vec<T>> {
        if self.next.is_empty() {
            return vec![vec![self.item]];
        }

        let mut vs = vec![];
        for n in self.next {
            for mut v in n.into_flatten() {
                v.push(self.item.clone());
                vs.push(v);
            }
        }
        vs
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ExpectTree<T> {
    path: Vec<usize>,
    tree: Vec<Expect<T>>,
}

impl<T> ExpectTree<T>
where
    T: Eq + PartialEq + Clone,
{
    /// Construct an empty tree
    pub fn new() -> Self {
        Self {
            path: vec![],
            tree: vec![],
        }
    }

    /// Register `value` as expected at this stage, and advance to the next token.
    pub fn advance(&mut self, value: T) {
        let mut r = &mut self.tree;

        for idx in &self.path {
            r = &mut r[*idx].next;
        }

        let mut idx = 0;
        for s in r.iter() {
            if s.item == value {
                self.path.push(idx);
                return;
            }
            idx += 1;
        }

        r.push(Expect {
            item: value,
            next: vec![],
        });
        self.path.push(idx);
    }

    #[cfg(test)]
    fn path_len(&self) -> usize {
        self.path.len()
    }

    /// Back track one token.
    pub fn retract_one(&mut self) {
        self.path.truncate(self.path.len() - 1);
    }

    /// Back track to a certain length.
    pub fn retract_path(&mut self, old_len: usize) {
        assert!(old_len <= self.path.len());
        self.path.truncate(old_len);
    }

    /// Resolve the tree into a list of possible token vectors, based on
    /// where that were junctions in the expectation tree.
    pub fn into_flatten(self) -> Vec<Vec<T>> {
        let mut tv = vec![];

        for v in self.tree {
            for mut t in v.into_flatten() {
                t.reverse();
                tv.push(t)
            }
        }

        tv
    }

    /// Return a reference to the last added token.
    pub fn last(&self) -> Option<&T> {
        let mut r = &self.tree;
        let mut item = None;

        for idx in &self.path {
            item = Some(&r[*idx].item);
            r = &r[*idx].next;
        }

        item
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn main() {
        use super::*;

        let mut et = ExpectTree::new();
        assert_eq!(et.last(), None);

        et.advance(2);
        assert_eq!(et.clone().into_flatten(), vec![vec![2]]);
        assert_eq!(et.last(), Some(&2));

        et.advance(3);
        assert_eq!(et.clone().into_flatten(), vec![vec![2, 3]]);
        assert_eq!(et.last(), Some(&3));

        et.retract_path(et.path_len() - 1);
        assert_eq!(et.last(), Some(&2));

        et.advance(3);
        assert_eq!(et.clone().into_flatten(), vec![vec![2, 3]]);

        et.retract_path(et.path_len() - 1);
        et.advance(4);
        assert_eq!(et.clone().into_flatten(), vec![vec![2, 3], vec![2, 4]]);

        et.retract_path(et.path_len() - 2);
        et.advance(1);
        assert_eq!(
            et.clone().into_flatten(),
            vec![vec![2, 3], vec![2, 4], vec![1]]
        );

        et.retract_path(et.path_len() - 1);
        et.advance(2);
        et.advance(3);
        et.advance(6);
        assert_eq!(
            et.clone().into_flatten(),
            vec![vec![2, 3, 6], vec![2, 4], vec![1]]
        );

        et.retract_path(et.path_len() - 1);
        et.advance(7);
        assert_eq!(
            et.clone().into_flatten(),
            vec![vec![2, 3, 6], vec![2, 3, 7], vec![2, 4], vec![1]]
        );
    }
}
