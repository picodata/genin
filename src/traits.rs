// Pass self to function and return new type
pub trait MapSelf<S>
where
    Self: Sized,
{
    type Target;
    type Error;

    fn map_self<F>(self, func: F) -> Result<Self::Target, Self::Error>
    where
        F: FnOnce(Self) -> Result<Self::Target, Self::Error>;
}

// Map Task to new type
pub trait Functor {
    type Unwrapped;
    type Wrapped<U>: Functor;
    type Error;

    fn map<F, U>(self, func: F) -> Result<Self::Wrapped<U>, Self::Error>
    where
        F: FnOnce(Self::Unwrapped) -> Result<U, Self::Error>;
}


