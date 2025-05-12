use std::iter::{empty, once};

pub trait ToConstraints {
    fn to_constraints(&self) -> impl Iterator<Item = &[usize]>;
}

impl ToConstraints for &[usize] {
    fn to_constraints(&self) -> impl Iterator<Item = &[usize]> {
        once(*self)
    }
}

impl<T: ToConstraints> ToConstraints for &[T]
{
    fn to_constraints(&self) -> impl Iterator<Item = &[usize]> {
        self.iter().flat_map(|x| x.to_constraints())
    }
}

impl<T: ToConstraints, const N: usize> ToConstraints for [T; N]
{
    fn to_constraints(&self) -> impl Iterator<Item = &[usize]> {
        self.iter().flat_map(|x| x.to_constraints())
    }
}

impl<T: ToConstraints, const N: usize> ToConstraints for &[T; N]
{
    fn to_constraints(&self) -> impl Iterator<Item = &[usize]> {
        self.iter().flat_map(|x| x.to_constraints())
    }
}

macro_rules! impl_to_contraints {
    ($($t:ident),* ; $($i:tt),*) => {
        impl<$($t: ToConstraints),*> ToConstraints for ($($t,)*) {
            fn to_constraints(&self) -> impl Iterator<Item = &[usize]> {
                empty()$(.chain(self.$i.to_constraints()))*
            }
        }
    };
}

impl_to_contraints!(;);
impl_to_contraints!(A ; 0);
impl_to_contraints!(A, B ; 0, 1);
impl_to_contraints!(A, B, C ; 0, 1, 2);
impl_to_contraints!(A, B, C, D ; 0, 1, 2, 3);
impl_to_contraints!(A, B, C, D, E ; 0, 1, 2, 3, 4);
impl_to_contraints!(A, B, C, D, E, F ; 0, 1, 2, 3, 4, 5);
impl_to_contraints!(A, B, C, D, E, F, G ; 0, 1, 2, 3, 4, 5, 6);
impl_to_contraints!(A, B, C, D, E, F, G, H ; 0, 1, 2, 3, 4, 5, 6, 7);
impl_to_contraints!(A, B, C, D, E, F, G, H, I ; 0, 1, 2, 3, 4, 5, 6, 7, 8);
impl_to_contraints!(A, B, C, D, E, F, G, H, I, J ; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
impl_to_contraints!(A, B, C, D, E, F, G, H, I, J, K ; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
impl_to_contraints!(A, B, C, D, E, F, G, H, I, J, K, L ; 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
