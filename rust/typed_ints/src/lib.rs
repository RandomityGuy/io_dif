use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Clone, Debug)]
pub struct TypedEnumerate<I, E: Copy + AddAssign<usize>> {
    iter: I,
    count: E,
}

impl<I, E: Copy + AddAssign<usize>> TypedEnumerate<I, E> {
    pub fn new(iter: I, first: E) -> TypedEnumerate<I, E> {
        TypedEnumerate { iter, count: first }
    }
}

impl<I, E: Copy + AddAssign<usize>> Iterator for TypedEnumerate<I, E>
where
    I: Iterator,
{
    type Item = (E, <I as Iterator>::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let a = self.iter.next()?;
        let i = *&self.count;
        self.count += 1;
        Some((i, a))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
    fn count(self) -> usize {
        self.iter.count()
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let a = self.iter.nth(n)?;
        // Possible undefined overflow.
        self.count += n;
        let i = *&self.count;
        self.count += 1;
        Some((i, a))
    }
}

pub trait TypedEnum<E: Copy + AddAssign<usize>>
where
    Self: Sized,
{
    fn typed_enumerate(self, first: E) -> TypedEnumerate<Self, E> {
        TypedEnumerate::<Self, E>::new(self, first)
    }
}

impl<I: Iterator, E: Copy + AddAssign<usize>> TypedEnum<E> for I where I: Sized {}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct TypedInt<B, X>(B, PhantomData<X>)
where
    B: Copy;

impl<B, X> Copy for TypedInt<B, X> where B: Copy {}

impl<B, X> Clone for TypedInt<B, X>
where
    B: Copy,
{
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<B, X> From<B> for TypedInt<B, X>
where
    B: Copy,
{
    fn from(inner: B) -> Self {
        Self(inner, PhantomData)
    }
}

impl<B, X> TypedInt<B, X>
where
    B: Copy,
{
    pub fn new(inner: B) -> Self {
        Self::from(inner)
    }
    pub fn into_inner(self) -> B {
        self.0
    }
    pub fn inner(&self) -> &B {
        &self.0
    }
    pub fn inner_mut(&mut self) -> &mut B {
        &mut self.0
    }
}

impl<B, A, X> Add<A> for TypedInt<B, X>
where
    B: Add<A, Output = B> + Copy,
{
    type Output = Self;
    fn add(self, rhs: A) -> Self::Output {
        Self(self.0.add(rhs), PhantomData)
    }
}

impl<B, A, X> AddAssign<A> for TypedInt<B, X>
where
    B: AddAssign<A> + Copy,
{
    fn add_assign(&mut self, rhs: A) {
        self.0.add_assign(rhs);
    }
}

impl<B, A, X> Sub<A> for TypedInt<B, X>
where
    B: Sub<A, Output = B> + Copy,
{
    type Output = Self;
    fn sub(self, rhs: A) -> Self::Output {
        Self(self.0.sub(rhs), PhantomData)
    }
}

impl<B, A, X> SubAssign<A> for TypedInt<B, X>
where
    B: SubAssign<A> + Copy,
{
    fn sub_assign(&mut self, rhs: A) {
        self.0.sub_assign(rhs);
    }
}

//impl<B, A, X> PartialOrd<A> for TypedInt<B, X> where B: PartialOrd<A>+Copy {
//    fn partial_cmp(&self, other: &A) -> Option<Ordering> {
//        self.0.partial_cmp(other)
//    }
//}
//
//impl<B, A, X> PartialEq<A> for TypedInt<B, X> where B: PartialEq<A>+Copy {
//    fn eq(&self, other: &A) -> bool {
//        self.0.eq(other)
//    }
//    fn ne(&self, other: &A) -> bool {
//        self.0.ne(other)
//    }
//}
//
//impl<B, X> Ord for TypedInt<B, X> where B: PartialOrd<TypedInt<B, X>>+Eq+Ord+Copy {
//    fn cmp(&self, other: &TypedInt<B, X>) -> Ordering {
//        self.0.cmp(&other.0)
//    }
//}
//
//impl<B, X> Eq for TypedInt<B, X> where B: PartialEq<TypedInt<B, X>>+Copy {
//}
//

#[macro_export]
macro_rules! typed_int {
    ($name:ident, $tag:ident, $base:ty) => {
        #[derive(Debug, Eq, Ord, PartialOrd, PartialEq)]
        pub struct $tag(usize);
        pub type $name = TypedInt<$base, $tag>;
    };
}
