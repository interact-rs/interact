use rand::distributions::Distribution;
use rand::distributions::Standard;
use rand::Rng;

pub trait Rand {
    fn new_random<R: Rng>(rng: &mut R) -> Self;
}

impl<T> Rand for T
where
    Standard: Distribution<T>,
{
    fn new_random<R: Rng>(rng: &mut R) -> Self {
        rng.gen()
    }
}
