use interact_derive::derive_interact_prelude;

derive_interact_prelude! {
    enum Option<T> {
        None,
        Some(T),
    }
}

derive_interact_prelude! {
    enum Result<T, E> {
        Ok(T),
        Err(E),
    }
}
