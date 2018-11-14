use crate::deser::{Deser, Error, Result, Tracker};
use crate::tokens::{Token, TokenInner};

macro_rules! impl_unsigned {
    ($a:tt) => {
        impl Deser for $a {
            fn deser<'a, 'b>(tracker: &mut Tracker<'a, 'b>) -> Result<Self> {
                if !tracker.has_remaining() {
                    return Err(Error::EndOfTokenList);
                }

                if let TokenInner::NonNegativeDecimal(nnd) = tracker.top().inner {
                    if nnd > u64::from(Self::max_value()) {
                        return Err(Error::NumberTooLarge);
                    }

                    tracker.step();
                    return Ok(nnd as Self);
                }
                if let TokenInner::Decimal(nnd) = tracker.top().inner {
                    if nnd < 0 {
                        return Err(Error::NumberTooSmall);
                    }
                    if nnd as u64 > u64::from(Self::max_value()) {
                        return Err(Error::NumberTooLarge);
                    }

                    tracker.step();
                    return Ok(nnd as Self);
                }

                Err(Error::UnexpectedToken)
            }
        }
    };
}

impl_unsigned!(u64);
impl_unsigned!(u32);
impl_unsigned!(u16);
impl_unsigned!(u8);

macro_rules! impl_signed {
    ($a:tt) => {
        impl Deser for $a {
            fn deser<'a, 'b>(tracker: &mut Tracker<'a, 'b>) -> Result<Self> {
                if !tracker.has_remaining() {
                    return Err(Error::EndOfTokenList);
                }

                if let TokenInner::NonNegativeDecimal(nnd) = tracker.top().inner {
                    if nnd >= Self::max_value() as u64 {
                        return Err(Error::NumberTooLarge);
                    }
                    tracker.step();
                    return Ok(nnd as Self);
                } else if let TokenInner::Decimal(dec) = tracker.top().inner {
                    if dec < Self::min_value() as i64 {
                        return Err(Error::NumberTooSmall);
                    }
                    if dec > Self::max_value() as i64 {
                        return Err(Error::NumberTooLarge);
                    }

                    tracker.step();
                    return Ok(dec as Self);
                }

                Err(Error::UnexpectedToken)
            }
        }
    };
}

impl_signed!(isize);
impl_signed!(i64);
impl_signed!(i32);
impl_signed!(i16);
impl_signed!(i8);

impl Deser for usize {
    fn deser<'a, 'b>(tracker: &mut Tracker<'a, 'b>) -> Result<Self> {
        if !tracker.has_remaining() {
            return Err(Error::EndOfTokenList);
        }

        if let TokenInner::NonNegativeDecimal(nnd) = tracker.top().inner {
            if nnd > Self::max_value() as u64 {
                return Err(Error::NumberTooLarge);
            }

            tracker.step();
            return Ok(nnd as Self);
        }
        if let TokenInner::Decimal(nnd) = tracker.top().inner {
            if nnd < 0 {
                return Err(Error::NumberTooSmall);
            }
            if nnd as u64 > Self::max_value() as u64 {
                return Err(Error::NumberTooLarge);
            }

            tracker.step();
            return Ok(nnd as Self);
        }

        Err(Error::UnexpectedToken)
    }
}

macro_rules! impl_simple {
    ($a:tt, $token:ident) => {
        impl Deser for $a {
            fn deser<'a, 'b>(tracker: &mut Tracker<'a, 'b>) -> Result<Self> {
                if !tracker.has_remaining() {
                    return Err(Error::EndOfTokenList);
                }

                if let TokenInner::$token(s) = &tracker.top().inner {
                    let s = s.clone();
                    tracker.step();
                    return Ok(s);
                }

                Err(Error::UnexpectedToken)
            }
        }
    };
}

impl_simple!(String, String);
impl_simple!(char, Char);

impl Deser for bool {
    fn deser<'a, 'b>(tracker: &mut Tracker<'a, 'b>) -> Result<Self> {
        let values = [("false", false), ("true", true)];

        if !tracker.has_remaining() {
            for (s, _) in values.iter() {
                tracker.possible_token(Token::new_borrowed(TokenInner::Ident, s));
            }
            return Err(Error::EndOfTokenList);
        }

        if let TokenInner::Ident = tracker.top().inner {
            for (s, value) in values.iter() {
                if *s == tracker.top().text {
                    tracker.step();
                    return Ok(*value);
                }
            }

            for (s, _) in values.iter() {
                if s.starts_with(tracker.top().text.as_ref()) {
                    tracker.possible_token(Token::new_borrowed(TokenInner::Ident, s));
                }
            }
        }

        Err(Error::UnexpectedToken)
    }
}
