#![recursion_limit = "256"]
extern crate proc_macro;
extern crate serde_derive_internals;
extern crate syn;

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, TokenNode, TokenTree};
use quote::Tokens;
use std::collections::{BTreeMap, HashSet};
use std::process::Command;
use syn::Meta::{List, Word};
use syn::NestedMeta::Meta;
use syn::{Data, DeriveInput, Fields, GenericParam, Generics};

#[macro_use]
extern crate quote;

struct DeriveInfo {
    from_interact: bool,
    opaque: bool,
    basic: bool,
}

#[proc_macro_derive(Interact, attributes(interact))]
pub fn derive_interact(input: TokenStream) -> TokenStream {
    derive_interact_inner(
        input,
        DeriveInfo {
            from_interact: false,
            opaque: false,
            basic: false,
        },
    )
}

#[proc_macro]
pub fn derive_interact_prelude(input: TokenStream) -> TokenStream {
    derive_interact_inner(
        input,
        DeriveInfo {
            from_interact: true,
            opaque: false,
            basic: true,
        },
    )
}

#[proc_macro]
pub fn derive_interact_opaque(input: TokenStream) -> TokenStream {
    derive_interact_inner(
        input,
        DeriveInfo {
            from_interact: true,
            opaque: true,
            basic: false,
        },
    )
}

#[proc_macro]
pub fn derive_interact_extern_opqaue(input: TokenStream) -> TokenStream {
    derive_interact_inner(
        input,
        DeriveInfo {
            from_interact: false,
            opaque: true,
            basic: false,
        },
    )
}

#[proc_macro]
pub fn derive_interact_basic(input: TokenStream) -> TokenStream {
    derive_interact_inner(
        input,
        DeriveInfo {
            from_interact: true,
            opaque: true,
            basic: true,
        },
    )
}

fn derive_interact_inner(input: TokenStream, info: DeriveInfo) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    let name = input.ident;

    let expanded = inner_derive_interact(input, info);

    if let Some((_, value)) =
        std::env::vars().find(|(key, _)| key.as_str() == "INTERACT_DERIVE_SAVE_DIR")
    {
        let dir = std::path::Path::new(value.as_str());
        tokens_to_rustfmt_file(&dir.join(format!("interact_derive_{}.rs", name)), &expanded);
    }

    expanded.into()
}

fn inner_derive_interact(input: DeriveInput, info: DeriveInfo) -> Tokens {
    let name = input.ident;
    let kr = if info.from_interact {
        quote! { crate }
    } else {
        quote! { crate::interact }
    };

    let (mut_assign, deser_impls) = impls_for_deser(&kr, &input, &info);
    let access_impls = impls_for_access(&input, mut_assign);
    let reflect_impls = impls_for_reflect(&input, &info);
    let uses = if info.basic {
        quote! {}
    } else {
        quote! {use super::#name;}
    };
    let module_name = syn::Ident::from(format!("impls_for_{}", name));

    quote! {
        mod #module_name {
            #uses

            use #kr::*;

            #access_impls
            #reflect_impls
            #deser_impls
        }
    }
}

#[derive(Debug)]
struct Function {
    mutability: Mutability,
    name: String,
    args: Vec<String>,
}

type FuncMap = BTreeMap<String, Function>;

fn fill_skip_bound_from_attr(attribute: Option<TokenTree>, skip_bound_set: &mut HashSet<String>) {
    let tt = if let Some(tt) = attribute {
        tt
    } else {
        panic!("Expected a token tree after skip_bound term");
    };

    if let TokenTree {
        kind: TokenNode::Group(Delimiter::Parenthesis, ts),
        ..
    } = tt
    {
        let mut ts = ts.clone().into_iter();
        if let Some(TokenTree {
            kind: TokenNode::Term(s),
            ..
        }) = ts.next()
        {
            skip_bound_set.insert(String::from(s.as_str()));
        } else {
            panic!("Expected term in attribute");
        }
    }
}

fn get_attr_info(
    attribute: &syn::Attribute,
    map: &mut FuncMap,
    skip_bound_set: &mut HashSet<String>,
    mut_assign: &mut bool,
) {
    for func in attribute.tts.clone().into_iter() {
        let ts = match func {
            TokenTree {
                kind: TokenNode::Op('=', _),
                ..
            }
            | TokenTree {
                kind: TokenNode::Literal(_),
                ..
            } => {
                // Skip comments
                continue;
            }
            TokenTree {
                kind: TokenNode::Group(Delimiter::Parenthesis, ts),
                ..
            } => ts,
            _ => {
                panic!("extend () in `interact` data type attribute `{:?}`", func);
            }
        };

        let mut ts = ts.clone().into_iter();
        let mutability_term = if let Some(TokenTree {
            kind: TokenNode::Term(mutability_term),
            ..
        }) = ts.next()
        {
            mutability_term
        } else {
            panic!("expected mut_fn/immut_fn term in data type attribute")
        };

        let mutability = match mutability_term.as_str() {
            "mut_fn" => Mutability::ModifyAccess,
            "immut_fn" => Mutability::ReadAccess,
            "skip_bound" => {
                fill_skip_bound_from_attr(ts.next(), skip_bound_set);
                continue;
            }
            "mut_assign" => {
                *mut_assign = true;
                continue;
            }
            _ => panic!(
                "Invalid term {} in `{}`",
                mutability_term.as_str(),
                attribute.tts.to_string()
            ),
        };

        let fn_def = if let Some(fn_def) = ts.next() {
            fn_def
        } else {
            panic!(
                "Expected a token tree after mutability term in `{}`",
                attribute.tts.to_string()
            );
        };

        if let TokenTree {
            kind: TokenNode::Group(Delimiter::Parenthesis, ts),
            ..
        } = fn_def
        {
            let mut ts = ts.clone().into_iter();
            let fn_name = if let Some(TokenTree {
                kind: TokenNode::Term(s),
                ..
            }) = ts.next()
            {
                s
            } else {
                panic!("Expected term in attribute `{}`", attribute.tts.to_string());
            };

            let name = String::from(fn_name.as_str());
            let mut func = Function {
                name: name.clone(),
                mutability,
                args: vec![],
            };

            let fn_def = if let Some(fn_def) = ts.next() {
                fn_def
            } else {
                panic!(
                    "Expected parameter specification () after {} in `{}`",
                    fn_name.as_str(),
                    attribute.tts.to_string()
                );
            };

            if let TokenTree {
                kind: TokenNode::Group(Delimiter::Parenthesis, ts),
                ..
            } = fn_def
            {
                for tt in ts.clone().into_iter() {
                    if let TokenTree {
                        kind: TokenNode::Term(s),
                        ..
                    } = tt
                    {
                        func.args.push(String::from(s.as_str()));
                    } else if let TokenTree {
                        kind: TokenNode::Op(',', _),
                        ..
                    } = tt
                    {
                        /* Ok */
                    } else {
                        panic!("Unexpected parameter token {:?}", tt);
                    }
                }
            }

            if map.get(&name).is_some() {
                panic!("Duplicate name {}", name.as_str());
            }

            map.insert(name, func);
        }
    }
}

fn call_impls(fnmap: &FuncMap, mutability: Mutability) -> (Tokens, Vec<Tokens>) {
    let mut arms = vec![];
    let mut descs = vec![];

    for func in fnmap.values() {
        let name = &func.name;
        let name_ident = syn::Ident::from(name.as_str());
        let mut match_vec = vec![];
        let mut arg_vec = vec![];
        let mut arg_str_vec = vec![];

        for arg in &func.args {
            arg_str_vec.push(arg.as_str());
            match_vec.push(syn::token::Underscore::new(proc_macro2::Span::call_site()));
            arg_vec.push(syn::Ident::from(arg.as_str()));
        }
        let arg_vec_ref = &arg_vec;
        let match_vec = &match_vec;

        let mut call_impl = quote! {
            <(#(#match_vec),*)>::deser(&mut _climber.borrow_tracker()).map(|(#(#arg_vec_ref),*)| {
                if !_climber.probe_only {
                    let _retval = self.#name_ident(#(#arg_vec_ref),*);
                    (_retcall)(&_retval, _climber);
                }
            }).map_err(CallError::Deser)
        };
        if mutability == Mutability::ReadAccess {
            if func.mutability == Mutability::ModifyAccess {
                call_impl = quote! {
                    <(#(#match_vec),*)>::deser(&mut _climber.borrow_tracker()).map(|(#(#arg_vec_ref),*)| {
                        if !_climber.probe_only {
                            // Non executing unsafe code only for coercing type checking
                            if false {
                                let mself : &mut Self = unsafe { std::mem::uninitialized() };
                                let _ = mself.#name_ident(#(#arg_vec_ref),*);
                            }
                        }
                    }).map_err(CallError::Deser)?;

                    return Err(CallError::NeedMutable);
                };
            }
        }

        arms.push(quote! {
            #name => {
                #call_impl
            }
        });

        descs.push(quote! {
            Function {
                name: #name,
                args: &[#(#arg_str_vec),*],
            }
        });
    }

    let arms = if arms.len() != 0 {
        quote! { #(#arms),* }
    } else {
        quote! {}
    };

    (
        quote! {
            match func_name {
                #arms
                _ => return Err(CallError::NoSuchFunction),
            }
        },
        descs,
    )
}

fn impls_for_access(input: &DeriveInput, mut mut_assign: bool) -> Tokens {
    let name = input.ident;
    let mut skip_bound_set = HashSet::new();
    let mut fnmap = BTreeMap::new();

    for attribute in &input.attrs {
        get_attr_info(attribute, &mut fnmap, &mut skip_bound_set, &mut mut_assign);
    }
    let generics = add_trait_bounds(
        input.generics.clone(),
        &skip_bound_set,
        &["Access", "Deser"],
    );
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let code = {
        quote! { Reflect::Direct(self) }
    };
    let code_mut = {
        quote! { ReflectMut::Direct(self) }
    };

    let (immut_call_impls, immut_call_desc) = call_impls(&fnmap, Mutability::ReadAccess);
    let (mut_call_impls, mut_call_desc) = call_impls(&fnmap, Mutability::ModifyAccess);

    let mut_assign_code = if mut_assign {
        quote! {
            fn mut_assign<'a, 'b>(
                &mut self,
                _tracker: &mut deser::Tracker<'a, 'b>,
                _probe_only: bool,
            ) -> Result<(), AssignError> {
                deser_assign(self, _tracker, _probe_only)
            }
        }
    } else {
        quote! {}
    };

    quote! {
        impl #impl_generics Access for #name #ty_generics #where_clause {
            fn immut_access(&self) -> ImmutAccess {
                ImmutAccess {
                    reflect: #code,
                    functions: &[#(#immut_call_desc),*],
                }
            }

            fn mut_access(&mut self) -> MutAccess {
                MutAccess {
                    reflect: #code_mut,
                    functions: &[#(#mut_call_desc),*],
                }
            }

            fn immut_call<'a>(&self, func_name: &'static str,
                              _climber: &mut Climber<'a>,
                              mut _retcall: RetValCallback<'a>) -> Result<(), CallError> {
                #immut_call_impls
            }

            fn mut_call<'a>(&mut self, func_name: &'static str,
                            _climber: &mut Climber<'a>,
                            mut _retcall: RetValCallback<'a>) -> Result<(), CallError> {
                #mut_call_impls
            }

            #mut_assign_code
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Mutability {
    ModifyAccess,
    ReadAccess,
}

fn impls_by_mutability(
    name: syn::Ident,
    data_fields: &Fields,
    in_enum: bool,
    mtype: Mutability,
) -> (Tokens, Tokens, Tokens, Tokens) {
    let mut named_match_arms: Vec<Tokens> = vec![];
    let mut index_match_arms: Vec<Tokens> = vec![];
    let desc: Tokens;
    let name = format!("{}", format!("{}", name));
    let params;

    let qmut = match mtype {
        Mutability::ReadAccess => quote! {},
        Mutability::ModifyAccess => quote! { mut },
    };

    match data_fields {
        Fields::Named(ref fields) => {
            let mut idents: Vec<syn::Ident> = vec![];
            let fnames: Vec<_> = fields
                .named
                .iter()
                .filter(|f| !is_skipped(&f.attrs))
                .map(|f| {
                    let ident = &f.ident;
                    idents.push(ident.unwrap().clone());

                    let f_i = if in_enum {
                        quote! { #ident }
                    } else {
                        quote! { & #qmut self.#ident }
                    };

                    (f_i, format!("{}", ident.as_ref().unwrap()))
                })
                .collect();

            let fnames_stringified: Vec<_> =
                fnames.iter().map(|(_, ref x)| quote! { #x }).collect();

            desc = quote! {
                Struct {
                    name: #name,
                    kind: StructKind::Fields(&[#(#fnames_stringified),*])
                }
            };

            named_match_arms = fnames
                .iter()
                .map(|(ref y, ref x)| {
                    quote! {
                        #x => Some(#y)
                    }
                })
                .collect();
            params = quote! {{#(#idents),*}};
        }
        Fields::Unnamed(ref fields) => {
            let mut idents: Vec<Tokens> = vec![];
            let mut i: usize = 0;

            let fnames: Vec<_> = fields
                .unnamed
                .iter()
                .filter(|f| !is_skipped(&f.attrs))
                .map(|_| {
                    let f_i = if in_enum {
                        let ident = syn::Ident::from(format!("f_{}", i));
                        quote! { #ident }
                    } else {
                        quote! { & #qmut self.#i }
                    };

                    idents.push(f_i.clone());

                    let r = (quote! { #i }, f_i);
                    i += 1;
                    r
                })
                .collect();

            let n = fnames.len();
            desc = quote! {
                Struct{
                    name: #name,
                    kind: StructKind::Tuple(#n)
                }
            };

            index_match_arms = fnames
                .iter()
                .map(|(idx, ref x)| {
                    quote! { #idx => Some(#x) }
                })
                .collect();
            params = quote! {(#(#idents),*)};
        }
        Fields::Unit => {
            desc = quote! {
                Struct {
                    name: #name,
                    kind: StructKind::Unit,
                }
            };
            params = quote! {};
        }
    }

    let named_match_arms = if named_match_arms.len() != 0 {
        quote! { #(#named_match_arms),*, }
    } else {
        quote! {}
    };

    let index_match_arms = if index_match_arms.len() != 0 {
        quote! { #(#index_match_arms),*, }
    } else {
        quote! {}
    };

    (named_match_arms, index_match_arms, desc, params)
}

fn impls_for_reflect(input: &DeriveInput, info: &DeriveInfo) -> Tokens {
    if info.opaque {
        return quote! {};
    }

    let name = input.ident;
    let generics = add_trait_bounds(input.generics.clone(), &HashSet::new(), &["Access"]);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let str_name = format!("{}", format!("{}", name));

    match &input.data {
        Data::Struct(data) => {
            let (named_match_arms, index_match_arms, desc, _) =
                impls_by_mutability(name.clone(), &data.fields, false, Mutability::ReadAccess);
            let (named_match_mut_arms, index_match_mut_arms, _, _) =
                impls_by_mutability(name.clone(), &data.fields, false, Mutability::ModifyAccess);

            quote! {
                impl #impl_generics ReflectStruct for #name #ty_generics #where_clause {
                    fn get_desc(&self) -> Struct {
                        #desc
                    }

                    fn get_field_by_name(&self, name: &'static str) -> Option<&dyn Access> {
                        match name {
                            #named_match_arms
                            _ => None,
                        }
                    }

                    fn get_field_by_idx(&self, idx: usize) -> Option<&dyn Access> {
                        match idx {
                            #index_match_arms
                            _ => None,
                        }
                    }

                    fn get_field_by_name_mut(&mut self, name: &'static str) -> Option<&mut dyn Access> {
                        match name {
                            #named_match_mut_arms
                            _ => None,
                        }
                    }

                    fn get_field_by_idx_mut(&mut self, idx: usize) -> Option<&mut dyn Access> {
                        match idx {
                            #index_match_mut_arms
                            _ => None,
                        }
                    }
                }

                impl #impl_generics ReflectDirect for #name #ty_generics #where_clause {
                    fn immut_reflector(&self, reflector: &std::sync::Arc<Reflector>) -> NodeTree {
                        Reflector::reflect_struct(reflector, &self.get_desc(), self, false)
                    }

                    fn immut_climber<'a>(
                        &self,
                        climber: &mut Climber<'a>,
                    ) -> Result<Option<NodeTree>, ClimbError> {
                        climber.check_field_access_immut(&EnumOrStruct::Struct(self))
                    }

                    fn mut_climber<'a>(
                        &mut self,
                        climber: &mut Climber<'a>,
                    ) -> Result<Option<NodeTree>, ClimbError> {
                        climber.check_field_access_mut(EnumOrStructMut::Struct(self))
                    }
                }
            }
        }
        Data::Enum(data) => {
            let mut field_by_name_match_arms = vec![];
            let mut field_by_idx_match_arms = vec![];
            let mut field_by_name_match_mut_arms = vec![];
            let mut field_by_idx_match_mut_arms = vec![];
            let mut desc_match_arms = vec![];
            let mut opt_names = vec![];

            for variant in data.variants.iter() {
                let ident = variant.ident;
                let variant_name = format!("{}", variant.ident);
                opt_names.push(variant_name);

                let (named_match_arms, index_match_arms, desc, params) = impls_by_mutability(
                    variant.ident.clone(),
                    &variant.fields,
                    true,
                    Mutability::ReadAccess,
                );
                let (named_match_mut_arms, index_match_mut_arms, _, _) = impls_by_mutability(
                    variant.ident.clone(),
                    &variant.fields,
                    true,
                    Mutability::ModifyAccess,
                );

                field_by_name_match_arms.push(quote! {
                    #name::#ident #params => {
                        match name {
                            #named_match_arms
                            _ => None,
                        }
                    }
                });

                field_by_idx_match_arms.push(quote! {
                    #name::#ident #params => {
                        match idx {
                            #index_match_arms
                            _ => None,
                        }
                    }
                });

                field_by_name_match_mut_arms.push(quote! {
                    #name::#ident #params => {
                        match name {
                            #named_match_mut_arms
                            _ => None,
                        }
                    }
                });

                field_by_idx_match_mut_arms.push(quote! {
                    #name::#ident #params => {
                        match idx {
                            #index_match_mut_arms
                            _ => None,
                        }
                    }
                });

                desc_match_arms.push(quote! {
                    #name::#ident #params => {
                        #desc
                    }
                });
            }

            let field_by_name_match_arms = if field_by_name_match_arms.len() != 0 {
                quote! { #(#field_by_name_match_arms),*, }
            } else {
                quote! { "_ => None"}
            };

            let field_by_idx_match_arms = if field_by_idx_match_arms.len() != 0 {
                quote! { #(#field_by_idx_match_arms),*, }
            } else {
                quote! { "_ => None"}
            };

            let field_by_name_match_mut_arms = if field_by_name_match_mut_arms.len() != 0 {
                quote! { #(#field_by_name_match_mut_arms),*, }
            } else {
                quote! { "_ => None"}
            };

            let field_by_idx_match_mut_arms = if field_by_idx_match_mut_arms.len() != 0 {
                quote! { #(#field_by_idx_match_mut_arms),*, }
            } else {
                quote! { "_ => None"}
            };

            let desc_match_arms = if desc_match_arms.len() != 0 {
                quote! { #(#desc_match_arms),*, }
            } else {
                quote! { "_ => panic!()"}
            };

            quote! {
                impl #impl_generics ReflectStruct for #name #ty_generics #where_clause {
                    fn get_desc(&self) -> Struct {
                        match self {
                            #desc_match_arms
                        }
                    }

                    fn get_field_by_name(&self, name: &'static str) -> Option<&dyn Access> {
                        match self {
                            #field_by_name_match_arms
                        }
                    }

                    fn get_field_by_idx(&self, idx: usize) -> Option<&dyn Access> {
                        match self {
                            #field_by_idx_match_arms
                        }
                    }

                    fn get_field_by_name_mut(&mut self, name: &'static str) -> Option<&mut dyn Access> {
                        match self {
                            #field_by_name_match_mut_arms
                        }
                    }

                    fn get_field_by_idx_mut(&mut self, idx: usize) -> Option<&mut dyn Access> {
                        match self {
                            #field_by_idx_match_mut_arms
                        }
                    }
                }

                impl #impl_generics ReflectEnum for #name #ty_generics #where_clause {
                    fn get_variant_desc(&self) -> Enum {
                        Enum {
                            name: #str_name,
                            opts: &[#(#opt_names),*],
                        }
                    }

                    fn get_variant_struct(&self) -> &dyn ReflectStruct {
                        self
                    }

                    fn get_variant_struct_mut(&mut self) -> &mut dyn ReflectStruct {
                        self
                    }
                }

                impl #impl_generics ReflectDirect for #name #ty_generics #where_clause {
                    fn immut_reflector(&self, reflector: &std::sync::Arc<Reflector>)
                        -> NodeTree
                    {
                        let p_struct = self.get_variant_struct();
                        let desc = p_struct.get_desc();
                        Reflector::reflect_struct(reflector, &desc, self, false)
                    }

                    fn immut_climber<'a>(
                        &self,
                        climber: &mut Climber<'a>,
                    ) -> Result<Option<NodeTree>, ClimbError> {
                        climber.check_field_access_immut(&EnumOrStruct::Enum(self))
                    }

                    fn mut_climber<'a>(
                        &mut self,
                        climber: &mut Climber<'a>,
                    ) -> Result<Option<NodeTree>, ClimbError> {
                        climber.check_field_access_mut(EnumOrStructMut::Enum(self))
                    }
                }
            }
        }
        _ => {
            panic!();
        }
    }
}

fn impl_struct_for_deser(name: Tokens, data_fields: &Fields, in_enum: bool) -> (Tokens, bool) {
    let name_str = format!("{}", format!("{}", name));
    let parse_name = if in_enum {
        quote! {}
    } else {
        quote! {
            let _name = Token::new_borrowed(TokenInner::Ident, #name_str);
            tracker.try_token(&_name)?;
        }
    };

    match data_fields {
        Fields::Named(ref fields) => {
            let mut match_arms = vec![];
            let mut match_arms_2 = vec![];
            let mut lets = vec![];
            let mut assigns = vec![];
            let mut names = vec![];

            for field in fields.named.iter() {
                if is_skipped(&field.attrs) {
                    return (
                        quote! {
                            return Err(deser::DeserError::Unbuildable);
                        },
                        false,
                    );
                }
                let ident = &field.ident;
                let ident_name = syn::Ident::from(format!("_assign_{}", ident.as_ref().unwrap()));
                let ident_str = format!("{}", ident.as_ref().unwrap());

                lets.push(quote! {
                    let mut #ident_name = None;
                });
                match_arms.push(quote! { #ident_str => Some(#ident_str) });
                match_arms_2.push(quote! {
                    Some(#ident_str) => {
                        if #ident_name.is_some() {
                            return Err(deser::DeserError::Unbuildable);
                        }
                        tracker.step();
                        tracker.try_token(&_colon)?;
                        #ident_name = Some(Deser::deser(tracker)?);
                    }
                });
                names.push(quote! { (#ident_str, #ident_name.is_some()) });
                assigns.push(quote! { #ident : #ident_name.unwrap() });
            }

            let expecting = assigns.len();
            let assigns = if assigns.is_empty() {
                quote! {}
            } else {
                quote! { #(#assigns),* }
            };

            let processing = if expecting > 0 {
                let names = &names;
                quote! {
                    #(#lets);*
                    let mut __expecting = #expecting;
                    while __expecting > 0 {
                        if !tracker.has_remaining() {
                            for (name, exclude) in &[#(#names,)*] {
                                if *exclude {
                                    continue;
                                }
                                tracker.possible_token(
                                    Token::new_borrowed(TokenInner::Ident, *name));
                            }
                            return Err(deser::DeserError::EndOfTokenList);
                        }
                        if let TokenInner::Ident = tracker.top().inner {
                            let opt = {
                                let text = tracker.top().text.as_ref();
                                match text {
                                    #(#match_arms),*,
                                    _ => None
                                }
                            };
                            match opt {
                                #(#match_arms_2),*,
                                _ => {
                                    let text = String::from(tracker.top().text.as_ref());
                                    for (name, exclude) in &[#(#names,)*] {
                                        if *exclude {
                                            continue;
                                        }
                                        if name.starts_with(&text) {
                                            tracker.possible_token(
                                                Token::new_borrowed(
                                                    TokenInner::Ident, *name));
                                        }
                                    }
                                    return Err(deser::DeserError::UnexpectedToken);
                                }
                            }
                            __expecting -= 1;
                            if __expecting == 0 {
                                tracker.try_token(&_curly_close)?;
                            } else {
                                tracker.try_token(&_comma)?;
                            }
                        } else {
                            return Err(deser::DeserError::UnexpectedToken);
                        }
                    }
                }
            } else {
                quote! {
                    tracker.try_token(&_culry_close)?;
                }
            };

            (
                quote! {
                    let _curly_open = Token::new_borrowed(TokenInner::CurlyOpen, " {");
                    let _curly_close = Token::new_borrowed(TokenInner::CurlyClose, "}");
                    let _comma = Token::new_borrowed(TokenInner::Comma, ", ");
                    let _colon = Token::new_borrowed(TokenInner::Colon, ": ");

                    #parse_name
                    tracker.try_token(&_curly_open)?;
                    #processing;

                    Ok(#name { #assigns })
                },
                true,
            )
        }
        Fields::Unnamed(ref fields) => {
            let mut assigns = vec![];

            for (idx, field) in fields.unnamed.iter().enumerate() {
                if is_skipped(&field.attrs) {
                    return (
                        quote! {
                            return Err(deser::DeserError::Unbuildable);
                        },
                        false,
                    );
                }

                let comma = if idx > 0 {
                    quote! { tracker.try_token(&_comma)?; }
                } else {
                    quote! {}
                };

                assigns.push(quote! {
                    {
                        #comma;
                        Deser::deser(tracker)?
                    }
                });
            }

            let assigns = if assigns.is_empty() {
                quote! {}
            } else {
                quote! { #(#assigns),* }
            };

            (
                quote! {
                    let _open = Token::new_borrowed(TokenInner::TupleOpen, "(");
                    let _close = Token::new_borrowed(TokenInner::TupleClose, ")");
                    let _comma = Token::new_borrowed(TokenInner::Comma, ", ");
                    let _colon = Token::new_borrowed(TokenInner::Colon, ": ");

                    #parse_name

                    tracker.try_token(&_open)?;
                    let rval = Ok(#name( #assigns ));
                    tracker.try_token(&_close)?;

                    rval
                },
                true,
            )
        }
        Fields::Unit => (
            quote! {
                #parse_name
                Ok(#name)
            },
            true,
        ),
    }
}

fn impls_for_deser(kr: &Tokens, input: &DeriveInput, info: &DeriveInfo) -> (bool, Tokens) {
    if info.opaque {
        return (false, quote! {});
    }

    let name = input.ident;
    let generics = add_trait_bounds(input.generics.clone(), &HashSet::new(), &["Deser"]);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let (res, active) = match &input.data {
        Data::Struct(data) => impl_struct_for_deser(quote! {#name}, &data.fields, false),
        Data::Enum(data) => {
            let mut names = vec![];
            let mut match_arms = vec![];

            for variant in data.variants.iter() {
                let ident = &variant.ident;
                let variant_name = format!("{}", variant.ident);
                let variant_access = if info.from_interact {
                    quote! {#ident}
                } else {
                    quote! {#name::#ident}
                };

                let (code, _) = impl_struct_for_deser(variant_access, &variant.fields, true);
                match_arms.push(quote! {
                    #variant_name => {
                        return {
                            tracker.step();
                            #code
                        }
                    }
                });

                names.push(quote! { #variant_name });
            }

            let names = &names;
            (
                quote! {
                    if !tracker.has_remaining() {
                        for name in &[#(#names,)*] {
                            tracker.possible_token(
                                Token::new_borrowed(TokenInner::Ident, *name));
                        }
                        return Err(deser::DeserError::EndOfTokenList);
                    }
                    if let TokenInner::Ident = tracker.top().inner {
                        let text = tracker.top().text.as_ref();
                        match text {
                            #(#match_arms),*
                            _ => {
                                let text = String::from(text);
                                for name in &[#(#names,)*] {
                                    if name.starts_with(&text) {
                                        tracker.possible_token(
                                            Token::new_borrowed(
                                                TokenInner::Ident, *name));
                                    }
                                }
                                return Err(deser::DeserError::UnexpectedToken);
                            }
                        }
                    } else {
                        return Err(deser::DeserError::UnexpectedToken);
                    }
                },
                true,
            )
        }
        _ => {
            panic!();
        }
    };

    (
        active,
        quote! {
            impl #impl_generics Deser for #name #ty_generics #where_clause {
                fn deser<'a, 'b>(tracker: &mut deser::Tracker<'a, 'b>) -> deser::Result<Self> {
                    use #kr::deser::*;
                    #res
                }
            }
        },
    )
}

fn get_interact_meta_items(attr: &syn::Attribute) -> Option<Vec<syn::NestedMeta>> {
    if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "interact" {
        match attr.interpret_meta() {
            Some(List(ref meta)) => Some(meta.nested.iter().cloned().collect()),
            _ => {
                // TODO: produce an error
                None
            }
        }
    } else {
        None
    }
}

fn is_skipped(attrs: &Vec<syn::Attribute>) -> bool {
    for meta_items in attrs.iter().filter_map(get_interact_meta_items) {
        for meta_item in meta_items {
            match meta_item {
                Meta(Word(word)) if word == "skip" => return true,
                _ => continue,
            }
        }
    }
    false
}

fn add_trait_bounds(
    mut generics: Generics,
    skip_set: &HashSet<String>,
    trait_names: &[&str],
) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            if let Some(_) = skip_set.get(&type_param.ident.to_string()) {
                continue;
            }
            for trait_name in trait_names {
                let trait_name = syn::Ident::from(*trait_name);
                let bound = syn::parse(quote! { #trait_name }.into()).unwrap();
                type_param.bounds.push(bound);
            }
        }
    }
    generics
}

fn tokens_to_rustfmt_file(filename: &std::path::Path, expanded: &Tokens) {
    let mut file = std::fs::File::create(&filename).unwrap();
    use std::io::Write;
    file.write_all(format!("{}", expanded).as_bytes()).unwrap();
    Command::new("rustfmt")
        .args(&[filename])
        .output()
        .expect("failed to execute process");
}
