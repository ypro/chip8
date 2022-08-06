use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Range;
use std::ops::RangeTo;
use std::ops::Add;
use num::Zero;
use std::marker::PhantomData;


#[derive(Copy, Clone)]
pub struct Array<T: Zero + Copy, const SIZE: usize, IDX=usize> {
    buf: [T; SIZE],
    index_type: PhantomData<IDX>,
}

// "Zero" trait requires "Add".
impl<T, const SIZE: usize> Add for Array<T, SIZE> 
where T: Zero + Copy + Add {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut new: Self = Self::new();

        for ((newref, left), right) in new.buf.iter_mut().zip(&self.buf).zip(&other.buf) {
            *newref = *left + *right;
        }
        new
    }
}

impl<T, const SIZE: usize> Zero for Array<T, SIZE> 
where T: Zero + Copy{
    fn zero() -> Self {
        Self {
            buf: [T::zero(); SIZE],
            index_type: PhantomData,
        }
    }

    fn is_zero(&self) -> bool {
        self.buf.iter().all(|&i| i.is_zero())
    }
}

impl<T: Zero + Copy, const SIZE: usize> Array<T, SIZE> {
    pub fn new() -> Self {
        Self {
            buf: [T::zero(); SIZE],
            index_type: PhantomData,
        }
    }
    pub fn iter(&self) -> impl Iterator<Item=&T> {
        self.buf.iter()
    }
    pub fn clear(&mut self) {
        self.buf = [T::zero(); SIZE];
    }
}

macro_rules! impl_index_slice {
    ($t:ty) => {
        impl <T: Zero + Copy, const SIZE: usize> Index<$t> for Array<T, SIZE> {
            type Output = [T];

            fn index(&self, i: $t) -> &Self::Output {
                self.buf.index(i)
            }
        }
    }
}

impl_index_slice!(Range<usize>);
impl_index_slice!(RangeTo<usize>);

macro_rules! impl_index {
    ($t:ty) => {
        impl <T: Zero + Copy, const SIZE: usize> Index<$t> for Array<T, SIZE> {
            type Output = T;

            fn index(&self, i: $t) -> &Self::Output {
                &self.buf[i as usize]
            }
        }

        impl <T: Zero + Copy, const SIZE: usize> IndexMut<$t> for Array<T, SIZE> {
            fn index_mut(&mut self, i: $t) -> &mut T {
                &mut self.buf[i as usize]
            }
        }

    };
}

impl_index!(u32);
impl_index!(u16);
impl_index!(u8);
impl_index!(usize);
impl_index!(i32);

#[cfg(test)]
mod tests {
    use super::Array;
    use num::Zero;

    #[test]
    fn arr_u8() {
        let mut a: Array<u8, 4> = Array::<u8, 4>::new();

        let _ = a[0..2];
        let _ = a[..2];
        a[0usize] = 0;
        a[1u32] = 1;
        a[2u8] = 2;
        a[3_i32] = 3;
        a[2] = 3u8;
        a[2u8] = 3;

        a.clear();
        assert!(a.is_zero());
    }
}
