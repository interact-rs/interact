use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

use crate::access::derive::{ReflectEnum, ReflectStruct, StructKind};
use crate::deser;
use crate::reflector::Reflector;
use crate::{
    Access, CallError, ExpectTree, Function, NodeInfo, NodeTree, ReflectMut, Token, TokenInner,
    TokenVec,
};

#[derive(Debug, Eq, PartialEq)]
pub enum ClimbError {
    AssignError(crate::access::AssignError),
    Borrowed,
    BorrowedMut,
    CallError(crate::access::CallError),
    DeserError(crate::deser::DeserError),
    Indirect,
    Locked,
    MissingStartComponent,
    NeedMutPath,
    NotFound,
    NullPath,
    UnattainedMutability,
    UnexpectedExpressionEnd,
    UnexpectedToken,
}

#[derive(Clone)]
pub struct Climber<'a> {
    pub probe_only: bool,
    reflector: Arc<Reflector>,
    pub expect: ExpectTree<Token<'static>>,
    pub tokenvec: TokenVec<'a>,
    pub valid_pos: usize,
    sender: Option<Sender<(Arc<Mutex<Climber<'static>>>, Result<NodeTree, ClimbError>)>>,
}

#[doc(hidden)]
pub enum EnumOrStruct<'a> {
    Enum(&'a ReflectEnum),
    Struct(&'a dyn ReflectStruct),
}

#[doc(hidden)]
pub enum EnumOrStructMut<'a, 'b> {
    Enum(&'b mut ReflectEnum),
    Struct(&'a mut dyn ReflectStruct),
}

macro_rules! if_mut {
    (mut, {$t: expr} else {$f:expr}) => {
        $t
    };
    (immut, {$t: expr} else {$f:expr}) => {
        $f
    };
    (mut, {$t: pat} else {$f:pat}) => {
        $t
    };
    (immut, {$t: pat} else {$f:pat}) => {
        $f
    };
}

macro_rules! climber_impl {
    (check_field_access, $mut:ident, $recurse:ident, $get_field_by_idx:ident,
     $get_field_by_name:ident, $general_access:ident, $get_variant_struct:ident, $EnumOrStruct:ident,
     $self:ident, $reflect:ident) =>
    { {
        let mut prefix : String = "".to_owned();

        enum T<A, B> { Struct(A), Enum(B) };

        let p_match = match if_mut!($mut, { &mut $reflect } else { $reflect }) {
            $EnumOrStruct::Struct(p_struct) => { T::Struct(p_struct) }
            $EnumOrStruct::Enum(p_enum) => { T::Enum(p_enum) }
        };

        if !$self.tokenvec.is_empty() {
            if let TokenInner::FieldAccess = &$self.tokenvec.top().inner {
                $self.tokenvec.advance(1);
            } else {
                $self.suggest_token(TokenInner::FieldAccess, Cow::Borrowed("."));
                return Ok(None);
            }

            if $self.tokenvec.remaining() >= 1 {
                if let TokenInner::Ident = &$self.tokenvec.top().inner {
                    prefix = String::from($self.tokenvec.top().text.as_ref());
                } else if let TokenInner::NonNegativeDecimal(nnd) = &$self.tokenvec.top().inner {
                    prefix = format!("{}", nnd);
                } else {
                    return Err(ClimbError::UnexpectedToken);
                }
            }
        } else {
            $self.suggest_token(TokenInner::FieldAccess, Cow::Borrowed("."));
        };

        match p_match {
            T::Struct(p_struct) => {
                let desc = p_struct.get_desc();

                match desc.kind {
                    StructKind::Tuple(i) => {
                        for j in 0..i {
                            let name = format!("{}", j);
                            if name == prefix {
                                let field =
                                    p_struct.$get_field_by_idx(j).unwrap();
                                $self.tokenvec.advance(1);
                                return $self.$general_access(
                                    if_mut!($mut, { &mut *field } else { &*field })
                                ).map(Some)
                            }
                            if name.starts_with(prefix.as_str()) {
                                $self.suggest_token(TokenInner::Ident, Cow::Owned(name));
                                $self.expect.retract_one();
                            }
                        }
                    },
                    StructKind::Fields(names) => {
                        for name in names {
                            if *name == prefix {
                                let field =
                                    p_struct.$get_field_by_name(name).unwrap();
                                $self.tokenvec.advance(1);
                                return $self.$general_access(
                                    if_mut!($mut, { &mut *field } else { &*field })
                                ).map(Some)
                            }
                            if name.starts_with(prefix.as_str()) {
                                $self.suggest_token(TokenInner::Ident, Cow::Borrowed(name));
                                $self.expect.retract_one();
                            }
                        }
                    }
                    StructKind::Unit => {},
                }
            }
            T::Enum(p_enum) => {
                let desc = p_enum.get_variant_struct().get_desc();
                let p_struct = p_enum.$get_variant_struct();
                if desc.name == prefix {
                    $self.tokenvec.advance(1);
                    if let Some(sub) = $self.$recurse(
                        if_mut!($mut, {
                            $EnumOrStruct::Struct(p_struct)
                        } else {
                            &$EnumOrStruct::Struct(p_struct)
                        })
                    )? {
                        return Ok(Some(sub));
                    } else {
                        return Ok(Some(Reflector::reflect_struct(&$self.reflector, &desc, p_struct, true)))
                    }
                } else {
                    if desc.name.starts_with(prefix.as_str()) {
                        $self.suggest_token(TokenInner::Ident, Cow::Borrowed(desc.name));
                        $self.expect.retract_one();
                    }
                }
            }
        }

        Ok(None)
    } };

    (check_functions, $mut:ident, $self:ident, $functions:ident, $dynvalue:ident, $immut_call:ident) =>
    { {
        if $functions.is_empty() {
            return Ok(None);
        }

        let mut prefix : String = "".to_owned();

        if !$self.tokenvec.is_empty() {
            if let TokenInner::FieldAccess = &$self.tokenvec.top().inner {
                $self.tokenvec.step();
                if !$self.tokenvec.is_empty() {
                    if let TokenInner::Ident = &$self.tokenvec.top().inner {
                        prefix = String::from($self.tokenvec.top().text.as_ref());
                    } else {
                        return Err(ClimbError::UnexpectedToken);
                    }
                }
            } else {
                return Ok(None);
            }
        } else {
            $self.suggest_token(TokenInner::FieldAccess, Cow::Borrowed("."));
        };

        for function in $functions {
            if function.name == prefix {
                $self.tokenvec.advance(1);
                if $self.tokenvec.is_empty() {
                    $self.suggest_token(TokenInner::TupleOpen, Cow::Borrowed("("));
                    return Ok(None);
                }

                if let TokenInner::TupleOpen = &$self.tokenvec.top().inner {
                    let retval : Rc<RefCell<Option<Result<Option<NodeTree>, ClimbError>>>> =
                        Rc::new(RefCell::new(None));
                    let retval2 = retval.clone();

                    let call_res = $dynvalue.$immut_call(
                        function.name, $self,
                        Box::new(move |access, climber| {
                            let mut retval = retval2.borrow_mut();
                            *retval = Some(climber.general_access_immut(access).map(Some));
                        }));

                    match call_res {
                        Err(CallError::NeedMutable) => {
                            if_mut!($mut, { {
                                return Err(ClimbError::UnattainedMutability);
                            } } else { {
                                return Err(ClimbError::NeedMutPath);
                            } });
                        }
                        Err(e) => {
                            return Err(ClimbError::CallError(e));
                        }
                        Ok(()) => {
                            match retval.borrow_mut().take(){
                                Some(retval) => { return retval }
                                None => {}
                            }
                            return Ok(None);
                        }
                    }
                } else {
                    return Err(ClimbError::UnexpectedToken);
                }
            }

            if function.name.starts_with(prefix.as_str()) {
                $self.suggest_token(TokenInner::Ident, Cow::Borrowed(function.name));
                $self.suggest_token(TokenInner::TupleOpen, Cow::Borrowed("("));
                $self.expect.retract_one();
                $self.expect.retract_one();
            }
        }

        Ok(None)
    } };

    (indirect_call, $mut:ident, $self:ident, $access:ident, $indirect:ident, $fname:ident) =>
    { {
          let mut climber =
              Climber {
                  probe_only: $self.probe_only,
                  reflector: $self.reflector.clone(),
                  expect: $self.expect.clone(),
                  valid_pos: $self.valid_pos,
                  sender: None,
                  tokenvec: $self.tokenvec.clone_owned(),
              };

          let recv = if $self.sender.is_none() {
              let (sender, receiver) = channel();
              climber.sender = Some(sender);
              Some(receiver)
          } else {
              climber.sender = $self.sender.take();
              None
          };

          let clone_ref = Arc::new(Mutex::new(climber));

          $access.$indirect(Box::new(move |access| {
              let clone_send = clone_ref.clone();
              let mut clone = clone_ref.lock().unwrap();
              let res = clone.$fname(access);
              if let Err(ClimbError::Indirect) = res {
                  return;
              } else {
                  let sender = clone.sender.take().unwrap();
                  let _ = sender.send((clone_send, res));
              }
          }));

          match recv {
              Some(recv) => {
                  let (clone_ref, res) = recv.recv().unwrap();
                  let mut clone = clone_ref.lock().unwrap();
                  std::mem::swap(&mut $self.expect, &mut clone.expect);
                  $self.tokenvec.take_pos(clone.tokenvec.pos());
                  $self.valid_pos = clone.valid_pos;
                  return res;
              }
              None => {
                  return Err(ClimbError::Indirect);
              }
          }
    } };
}

impl<'a> Climber<'a> {
    pub fn new(max_nodes: usize, probe_only: bool, tokens: &'a [Token<'a>]) -> Self {
        Self {
            probe_only,
            tokenvec: TokenVec::new(tokens),
            reflector: Reflector::new(max_nodes),
            expect: ExpectTree::new(),
            valid_pos: 0,
            sender: None,
        }
    }

    pub fn general_access_immut<'b>(
        &mut self,
        dynvalue: &'b dyn Access,
    ) -> Result<NodeTree, ClimbError> {
        use crate::Reflect::*;

        self.expect = ExpectTree::new();
        self.valid_pos = self.tokenvec.pos();

        if !self.tokenvec.is_empty() {
            if let TokenInner::Assign = &self.tokenvec.top().inner {
                return Err(ClimbError::NeedMutPath);
            }
        }

        let immut_access = dynvalue.immut_access();

        let save_tokenvec = self.tokenvec.clone();

        if let Some(opt_val) = self.check_functions_immut(immut_access.functions, dynvalue)? {
            return Ok(opt_val);
        }
        self.expect.retract_path(0);
        let pos = self.tokenvec.pos();
        self.tokenvec = save_tokenvec.clone();

        let pos = match immut_access.reflect {
            Direct(access) => {
                if let Some(opt_val) = access.immut_climber(self)? {
                    return Ok(opt_val);
                }
                self.expect.retract_path(0);
                let pos = std::cmp::max(self.tokenvec.pos(), pos);
                self.tokenvec = save_tokenvec.clone();
                pos
            }
            Indirect(access) => {
                climber_impl!(indirect_call, mut, self, access, indirect, general_access_immut);
            }
        };

        self.tokenvec.take_pos(pos);
        if self.tokenvec.remaining() > 0 {
            return Err(ClimbError::UnexpectedToken);
        }

        Ok(Reflector::reflect(&self.reflector, dynvalue))
    }

    pub fn general_access_mut<'b>(
        &mut self,
        dynvalue: &'b mut dyn Access,
    ) -> Result<NodeTree, ClimbError> {
        self.expect = ExpectTree::new();
        self.valid_pos = self.tokenvec.pos();

        if !self.tokenvec.is_empty() {
            if let TokenInner::Assign = &self.tokenvec.top().inner {
                self.tokenvec.advance(1);

                let probe_only = self.probe_only;
                let mut tracker = self.borrow_tracker();
                let res = dynvalue.mut_assign(&mut tracker, probe_only);
                return match res {
                    Ok(()) => {
                        self.valid_pos = self.tokenvec.pos();
                        Ok(NodeInfo::Leaf(Cow::Borrowed("")).into_node())
                    }
                    Err(e) => Err(ClimbError::AssignError(e)),
                };
            }
        }

        let functions = { dynvalue.mut_access().functions };

        let save_tokenvec = self.tokenvec.clone();
        if let Some(opt_val) = self.check_functions_mut(functions, dynvalue)? {
            return Ok(opt_val);
        }
        let pos = self.tokenvec.pos();
        self.tokenvec = save_tokenvec.clone();
        self.expect.retract_path(0);

        let pos = {
            let mut mut_access = dynvalue.mut_access();
            match mut_access.reflect {
                ReflectMut::Direct(ref mut access) => {
                    if let Some(opt_val) = access.mut_climber(self)? {
                        return Ok(opt_val);
                    }
                    self.expect.retract_path(0);
                    let pos = std::cmp::max(self.tokenvec.pos(), pos);
                    self.tokenvec = save_tokenvec.clone();
                    pos
                }
                ReflectMut::Indirect(access) => {
                    climber_impl!(indirect_call, mut, self, access, indirect_mut, general_access_mut);
                }
                _ => pos,
            }
        };

        self.tokenvec.take_pos(pos);

        if self.tokenvec.remaining() > 0 {
            return Err(ClimbError::UnexpectedToken);
        }

        Ok(Reflector::reflect(&self.reflector, dynvalue))
    }

    fn check_functions_immut<'b>(
        &mut self,
        functions: &'static [Function],
        dynvalue: &'b dyn Access,
    ) -> Result<Option<NodeTree>, ClimbError> {
        climber_impl!(
            check_functions,
            immut,
            self,
            functions,
            dynvalue,
            immut_call
        )
    }

    fn check_functions_mut<'b>(
        &mut self,
        functions: &'static [Function],
        dynvalue: &'b mut dyn Access,
    ) -> Result<Option<NodeTree>, ClimbError> {
        climber_impl!(check_functions, mut, self, functions, dynvalue, mut_call)
    }

    pub fn check_field_access_immut<'c, 'b>(
        &mut self,
        reflect: &'c EnumOrStruct<'b>,
    ) -> Result<Option<NodeTree>, ClimbError> {
        climber_impl!(
            check_field_access,
            immut,
            check_field_access_immut,
            get_field_by_idx,
            get_field_by_name,
            general_access_immut,
            get_variant_struct,
            EnumOrStruct,
            self,
            reflect
        )
    }

    pub fn check_field_access_mut<'c, 'b>(
        &mut self,
        mut reflect: EnumOrStructMut<'c, 'b>,
    ) -> Result<Option<NodeTree>, ClimbError> {
        climber_impl!(check_field_access, mut, check_field_access_mut,
                      get_field_by_idx_mut, get_field_by_name_mut, general_access_mut, get_variant_struct_mut,
                      EnumOrStructMut, self, reflect)
    }

    fn suggest_token(&mut self, inner: TokenInner, text: Cow<'static, str>) {
        self.expect.advance(Token {
            inner,
            space_diff: 0,
            text,
        });
    }

    pub fn mutex_handling<'b>(
        &mut self,
        m: &'b std::sync::Mutex<dyn Access + 'b>,
    ) -> Result<NodeTree, ClimbError> {
        let save = self.clone();
        let retval = {
            match m.try_lock() {
                Ok(locked) => self.general_access_immut(&*locked),
                Err(_) => return Err(ClimbError::Locked),
            }
        };

        if let Err(ClimbError::NeedMutPath) = &retval {
            *self = save;
            match m.try_lock() {
                Ok(mut locked) => return self.general_access_mut(&mut *locked),
                Err(_) => return Err(ClimbError::Locked),
            }
        } else {
            return retval;
        }
    }

    pub fn refcell_handling<'b>(
        &mut self,
        m: &'b std::cell::RefCell<dyn Access + 'b>,
    ) -> Result<NodeTree, ClimbError> {
        let save = self.clone();
        let retval = {
            match m.try_borrow() {
                Ok(borrowed) => self.general_access_immut(&*borrowed),
                Err(_) => return Err(ClimbError::Borrowed),
            }
        };
        if let Err(ClimbError::NeedMutPath) = &retval {
            *self = save;
            match m.try_borrow_mut() {
                Ok(mut borrowed) => self.general_access_mut(&mut *borrowed),
                Err(_) => return Err(ClimbError::BorrowedMut),
            }
        } else {
            return retval;
        }
    }

    pub fn open_bracket(&mut self) -> bool {
        if self.tokenvec.is_empty() {
            self.suggest_token(TokenInner::SubscriptOpen, Cow::Borrowed("["));
            return false;
        }

        if let TokenInner::SubscriptOpen = &self.tokenvec.top().inner {
            self.tokenvec.advance(1);
            true
        } else {
            false
        }
    }

    pub fn close_bracket(&mut self) -> Result<(), ClimbError> {
        if self.tokenvec.is_empty() {
            self.suggest_token(TokenInner::TupleClose, Cow::Borrowed("]"));
            return Err(ClimbError::UnexpectedExpressionEnd);
        }

        if let TokenInner::SubscriptClose = &self.tokenvec.top().inner {
            self.tokenvec.advance(1);
            Ok(())
        } else {
            Err(ClimbError::UnexpectedToken)
        }
    }

    pub fn borrow_tracker<'b>(&'b mut self) -> deser::Tracker<'a, 'b> {
        deser::Tracker::new(&mut self.expect, &mut self.tokenvec)
    }
}
