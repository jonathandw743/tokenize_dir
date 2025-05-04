pub trait ToIter {
    fn to_iter(&self) -> impl Iterator<Item = impl AsRef<[usize]>>;

    fn file_indices(&self) -> Vec<usize> {
        crate::file_indices(self.to_iter())
    }
}

impl ToIter for &[&[usize]] {
    fn to_iter(&self) -> impl Iterator<Item = impl AsRef<[usize]>> {
        self.iter()
    }
}

impl<const N: usize> ToIter for [&[usize]; N] {
    fn to_iter(&self) -> impl Iterator<Item = impl AsRef<[usize]>> {
        self.iter()
    }
}

impl<const N: usize> ToIter for &[&[usize]; N] {
    fn to_iter(&self) -> impl Iterator<Item = impl AsRef<[usize]>> {
        self.iter()
    }
}

impl<T> ToIter for &[T]
where
    T: ToIter,
{
    fn to_iter(&self) -> impl Iterator<Item = impl AsRef<[usize]>> {
        self.iter().flat_map(|x| x.to_iter())
    }
}

impl<T, const N: usize> ToIter for [T; N]
where
    T: ToIter,
{
    fn to_iter(&self) -> impl Iterator<Item = impl AsRef<[usize]>> {
        self.iter().flat_map(|x| x.to_iter())
    }
}

impl<T, const N: usize> ToIter for &[T; N]
where
    T: ToIter,
{
    fn to_iter(&self) -> impl Iterator<Item = impl AsRef<[usize]>> {
        self.iter().flat_map(|x| x.to_iter())
    }
}