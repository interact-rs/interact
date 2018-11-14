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
    pub fn new() -> Self {
        Self {
            path: vec![],
            tree: vec![],
        }
    }

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

    pub fn retract_one(&mut self) {
        self.path.truncate(self.path.len() - 1);
    }

    pub fn retract_path(&mut self, old_len: usize) {
        assert!(old_len <= self.path.len());
        self.path.truncate(old_len);
    }

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
