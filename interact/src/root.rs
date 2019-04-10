use std::collections::BTreeMap;

use crate::{
    tokens::parse_to_tokens, Access, Assist, ClimbError, Climber, NextOptions, NodeTree, Token,
};

/// Holds a root dictionary of `Send`-able trait objects that implement `Access` and are therefore
/// Interact-able. These are most likely objects that are held globally behind an `Arc`.
#[derive(Default)]
pub struct RootSend {
    pub owned: BTreeMap<&'static str, Box<dyn Access + Send>>,
}

impl RootSend {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn as_root(&mut self) -> Root {
        Root {
            send: Some(self),
            local: None,
        }
    }
}

/// Holds a root dictionary of trait objects that implement `Access` and are therefore Interact-able.
/// These are most likely objects that are held locally behind an `Rc`.
#[derive(Default)]
pub struct RootLocal {
    pub owned: BTreeMap<&'static str, Box<dyn Access>>,
}

impl RootLocal {
    pub fn new() -> Self {
        Default::default()
    }
}

/// A temporary binder of `RootSend` and `RootLocal` dictionaries, used for providing a unified
/// dictionary to the user.
pub struct Root<'a, 'b> {
    pub send: Option<&'a mut RootSend>,
    pub local: Option<&'b mut RootLocal>,
}

impl<'a, 'b> Root<'a, 'b> {
    /// Probe a path, checking if it is valid. If it contains a function name, it will not be
    /// called. If it contains an assignment, the assignment will not take place but the parameters
    /// value will check for `Deser` deserialization.
    ///
    /// This call may block the current thread until `ReflectIndirect` evaluation is resolved, and
    /// it may block because some fields, depending on the types and usage, could be behind a
    /// `Mutex` lock.
    pub fn probe(&mut self, path_str: &str) -> (Result<NodeTree, ClimbError>, Assist<String>) {
        self._access(path_str, true)
    }

    /// Perform evaluation of the provided path. This may perform assignments, or call user-defined
    /// functions via the `#[interact(...)]` type attribute.
    ///
    /// This call may block the current thread until `ReflectIndirect` evaluation is resolved, and
    /// it may block because some fields, depending on the types and usage, could be behind a
    /// `Mutex` lock.
    pub fn access(&mut self, path_str: &str) -> (Result<NodeTree, ClimbError>, Assist<String>) {
        self._access(path_str, false)
    }

    pub fn keys(&self) -> Vec<&'static str> {
        let mut v = vec![];
        match &self.send {
            None => {}
            Some(x) => {
                for k in x.owned.keys() {
                    v.push(*k);
                }
            }
        }
        match &self.local {
            None => {}
            Some(x) => {
                for k in x.owned.keys() {
                    v.push(*k);
                }
            }
        }

        v
    }

    fn _access(
        &mut self,
        path_str: &str,
        probe_only: bool,
    ) -> (Result<NodeTree, ClimbError>, Assist<String>) {
        enum Item<'a, 'b> {
            Send(&'a mut Box<dyn Access + Send>),
            Local(&'b mut Box<dyn Access>),
        };
        let mut h = std::collections::BTreeMap::new();
        match &mut self.send {
            None => {}
            Some(x) => {
                for (k, v) in x.owned.iter_mut() {
                    h.insert(*k, Item::Send(v));
                }
            }
        }
        match &mut self.local {
            None => {}
            Some(x) => {
                for (k, v) in x.owned.iter_mut() {
                    h.insert(*k, Item::Local(v));
                }
            }
        }
        let matching_prefix_keys = h
            .keys()
            .filter(|x| x.starts_with(path_str))
            .map(|x| String::from(&x[..]))
            .collect();

        let ret_assist = |valid| {
            let mut assist = Assist::default();
            let matching: Vec<_> = matching_prefix_keys;
            if !matching.is_empty() {
                assist.pend(valid)
            }
            assist.next_options(NextOptions::Avail(0, matching))
        };
        let tokens = match parse_to_tokens(path_str).map_err(ClimbError::TokenError) {
            Err(err) => {
                return (Err(err), ret_assist(0));
            }
            Ok(tokens) => tokens,
        };
        if tokens.is_empty() {
            return (Err(ClimbError::NullPath), ret_assist(0));
        }

        let first_token = tokens[0].text.as_ref();
        let item = match h.get(first_token) {
            Some(v) => v,
            None => {
                return (
                    Err(ClimbError::MissingStartComponent),
                    ret_assist(first_token.len()),
                );
            }
        };

        let start_pos = tokens[0].space_diff + tokens[0].text.len();
        let tokens = &tokens[1..];
        let mut climber = Climber::new(200, probe_only, tokens);
        let climber_clone = climber.clone();

        let mut res = match item {
            Item::Local(x) => match climber.general_access_immut(&***x) {
                Err(ClimbError::NeedMutPath) => match h.get_mut(first_token) {
                    Some(Item::Local(item)) => {
                        climber = climber_clone;
                        climber.general_access_mut(&mut ***item)
                    }
                    _ => {
                        return (
                            Err(ClimbError::MissingStartComponent),
                            ret_assist(first_token.len()),
                        );
                    }
                },
                e => e,
            },
            Item::Send(x) => match climber.general_access_immut(&***x) {
                Err(ClimbError::NeedMutPath) => match h.get_mut(first_token) {
                    Some(Item::Send(item)) => {
                        climber = climber_clone;
                        climber.general_access_mut(&mut ***item)
                    }
                    _ => {
                        return (
                            Err(ClimbError::MissingStartComponent),
                            ret_assist(first_token.len()),
                        );
                    }
                },
                e => e,
            },
        };

        match &mut res {
            Ok(res) => {
                res.resolve();
            }
            Err(_) => {}
        }

        let (old_assist, pending_partial) = climber.convert_to_assist();

        // Convert the tokens-based Assist back to String-based assist
        let mut new_assist = Assist::default();

        let (valid, pending, pending_special, next_options) = old_assist.dismantle();

        let mut valid_len = start_pos;
        for token in tokens.iter().take(valid) {
            valid_len += token.space_diff;
            valid_len += token.text.len();
        }
        new_assist.pend(valid_len);
        new_assist.commit_pending();

        let mut pending_len = 0;
        let mut pending_avail = 0;
        let mut pending_special_len = 0;
        for (i, token) in tokens.iter().enumerate().skip(valid).take(pending) {
            let nr_pending = i - valid;
            pending_len += token.space_diff;
            pending_len += token.text.len();
            if let NextOptions::Avail(pos, _) = next_options {
                if nr_pending < pos {
                    pending_avail = pending_len;
                }
            }
            if i >= valid + pending - pending_special {
                pending_special_len += token.space_diff;
                pending_special_len += token.text.len();
            }
        }
        new_assist.pend(pending_len);
        new_assist.set_pending_special(pending_special_len);

        let conv = |v: Vec<Vec<Token<'static>>>| {
            let mut str_suggestions = vec![];
            for suggestion in v {
                let mut s = String::new();
                for token in suggestion {
                    let spaces = "             ";
                    let max_space = std::cmp::min(spaces.len(), token.space_diff);
                    s.push_str(&spaces[..max_space]);
                    s.push_str(token.text.as_ref());
                }
                str_suggestions.push(s);
            }
            str_suggestions
        };

        let next_options = match next_options {
            NextOptions::NoOptions => NextOptions::NoOptions,
            NextOptions::Avail(_, v) => {
                NextOptions::Avail(pending_avail - pending_partial, conv(v))
            }
        };

        (res, new_assist.next_options(next_options))
    }
}
