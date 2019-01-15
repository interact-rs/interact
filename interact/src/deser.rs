use crate::{ExpectTree, Token, TokenInner, TokenVec};

#[derive(Debug, Eq, PartialEq)]
pub enum DeserError {
    EndOfTokenList,
    NumberTooLarge,
    NumberTooSmall,
    UnexpectedToken,
    Unbuildable,
}

pub struct Tracker<'a, 'b> {
    pub expect: &'b mut ExpectTree<Token<'static>>,
    pub tokenvec: &'b mut TokenVec<'a>,
    steps: usize,
}

pub type Result<T> = std::result::Result<T, DeserError>;

impl<'a, 'b> Tracker<'a, 'b> {
    pub fn new(expect: &'b mut ExpectTree<Token<'static>>, tokenvec: &'b mut TokenVec<'a>) -> Self {
        Self {
            expect,
            tokenvec,
            steps: 0,
        }
    }

    pub fn possible_token(&mut self, token: Token<'static>) {
        self.expect.advance(token);
        self.expect.retract_one();
    }

    pub fn try_token(&mut self, token: &Token<'static>) -> Result<bool> {
        if !self.tokenvec.has_remaining() {
            let mut token = token.clone();
            if let Some(last) = self.expect.last() {
                if let TokenInner::Comma = &last.inner {
                    if last.space_suffix() == 0 {
                        token.space_diff += 1;
                    }
                }
            } else if self.steps == 0 {
                token.space_diff += 1;
            }
            self.expect.advance(token);
            Ok(false)
        } else if self.tokenvec.top().similar(token) {
            self.step();
            Ok(true)
        } else if self.tokenvec.top().is_prefix_of(token) {
            self.expect.advance(token.clone());
            Ok(false)
        } else {
            Err(DeserError::UnexpectedToken)
        }
    }

    pub fn has_remaining(&self) -> bool {
        self.tokenvec.has_remaining()
    }

    pub fn top(&self) -> &Token<'a> {
        self.tokenvec.top()
    }

    pub fn step(&mut self) {
        *self.expect = ExpectTree::new();
        self.tokenvec.step();
        self.steps += 1;
    }
}

pub trait Deser: Sized {
    fn deser<'a, 'b>(_tracker: &mut Tracker<'a, 'b>) -> Result<Self> {
        return Err(DeserError::Unbuildable);
    }
}

mod basic;
mod btreemap;
mod derefs;
mod hashmap;
mod hashset;
mod instant;
mod mutex;
mod refcell;
mod tuple;
mod vec;
