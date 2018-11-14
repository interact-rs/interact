/// Given a list of items, this provides assistance for completing them.
#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub struct Assist<T> {
    /// How many of the first items are valid
    valid: usize,

    /// Following the valid items, how many more items will be valid once
    /// more items are added
    pending: usize,

    /// Among these pending items, how many are have special marking,
    /// from the end of the pending list.
    pending_special: usize,

    /// An optional list of items that can follow the valid+pending ones.
    next_options: NextOptions<T>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum NextOptions<T> {
    NoOptions,
    Avail(usize, Vec<T>),
}

impl<T> Default for NextOptions<T> {
    fn default() -> Self {
        NextOptions::NoOptions
    }
}

impl<T> NextOptions<T> {
    pub fn into_position(self, valid: usize) -> (usize, Vec<T>) {
        match self {
            NextOptions::NoOptions => (0, vec![]),
            NextOptions::Avail(pos, v) => (valid + pos, v),
        }
    }
}

impl<T> Assist<T> {
    pub fn pend(&mut self, count: usize) {
        self.pending += count;
    }

    pub fn pend_one(&mut self) {
        self.pend(1);
    }

    pub fn pending(&self) -> usize {
        self.pending
    }

    pub fn has_pending(&self) -> bool {
        self.pending > 0
    }

    pub fn commit_pending(&mut self) {
        self.valid += self.pending;
        self.pending = 0;
        self.pending_special = 0;
    }

    pub fn next_options(mut self, next_options: NextOptions<T>) -> Self {
        self.next_options = next_options;
        self
    }

    pub fn set_next_options(&mut self, next_options: NextOptions<T>) {
        self.next_options = next_options;
    }

    pub fn set_pending_special(&mut self, pending_special: usize) {
        self.pending_special = pending_special
    }

    pub fn with_valid(mut self, valid: usize) -> Self {
        self.valid += valid;
        self
    }

    pub fn dismantle(self) -> (usize, usize, usize, NextOptions<T>) {
        let Self {
            valid,
            pending,
            pending_special,
            next_options,
        } = self;

        (valid, pending, pending_special, next_options)
    }
}
