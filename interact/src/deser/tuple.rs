use crate::deser::{Deser, Result, Tracker};
use crate::tokens::{Token, TokenKind};

macro_rules! tuple {
    ({ $(($n:ident, $i:tt)),* }) => {
        impl<$($n),*> Deser for ($($n),*)
            where $($n : Deser),*
        {
            fn deser<'a, 'b>(tracker: &mut Tracker<'a, 'b>) -> Result<Self> {
                let open = Token::new_borrowed(TokenKind::TupleOpen, "(");
                let close = Token::new_borrowed(TokenKind::TupleClose, ")");
                let comma = Token::new_borrowed(TokenKind::Comma, ", ");

                tracker.try_token(&open)?;
                Ok(($(
                    {
                        let x : $n = Deser::deser(tracker)?;
                        if $i {
                            tracker.try_token(&close)?;
                        } else {
                            tracker.try_token(&comma)?;
                        }
                        x
                    }
                ),*))
            }
        }
    };
}

tuple!({(A, false), (B, true)});
tuple!({(A, false), (B, false), (C, true)});
tuple!({(A, false), (B, false), (C, false), (D, true)});
tuple!({(A, false), (B, false), (C, false), (D, false), (E, true)});
tuple!({(A, false), (B, false), (C, false), (D, false), (E, false), (F, true)});
tuple!({(A, false), (B, false), (C, false), (D, false), (E, false), (F, false), (G, true)});
tuple!({(A, false), (B, false), (C, false), (D, false), (E, false), (F, false), (G, false), (H, true)});
tuple!({(A, false), (B, false), (C, false), (D, false), (E, false), (F, false), (G, false), (H, false), (I, true)});
tuple!({(A, false), (B, false), (C, false), (D, false), (E, false), (F, false), (G, false), (H, false), (I, false), (J, true)});

impl<A> Deser for (A,)
where
    A: Deser,
{
    fn deser<'a, 'b>(tracker: &mut Tracker<'a, 'b>) -> Result<Self> {
        let open = Token::new_borrowed(TokenKind::TupleOpen, "(");
        let close = Token::new_borrowed(TokenKind::TupleClose, ")");
        tracker.try_token(&open)?;
        let a = Deser::deser(tracker)?;
        // TODO: allow for an extra ',' token
        tracker.try_token(&close)?;
        Ok((a,))
    }
}

impl Deser for () {
    fn deser<'a, 'b>(tracker: &mut Tracker<'a, 'b>) -> Result<Self> {
        let open = Token::new_borrowed(TokenKind::TupleOpen, "(");
        let close = Token::new_borrowed(TokenKind::TupleClose, ")");
        tracker.try_token(&open)?;
        tracker.try_token(&close)?;
        Ok(())
    }
}
