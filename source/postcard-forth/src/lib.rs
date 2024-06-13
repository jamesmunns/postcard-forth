#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![allow(clippy::result_unit_err, clippy::missing_safety_doc)]

use core::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

pub struct SerStream<'a> {
    cur: *mut u8,
    end: *mut u8,
    _plt: PhantomData<&'a mut u8>,
}

impl<'a> SerStream<'a> {
    pub fn push_one(&mut self, one: u8) -> Result<(), ()> {
        if self.cur == self.end {
            Err(())
        } else {
            unsafe { self.cur.write(one) }
            self.cur = self.cur.wrapping_add(1);
            Ok(())
        }
    }

    pub fn push_n(&mut self, sli: &[u8]) -> Result<(), ()> {
        let remain = self.remain();
        let n = sli.len();
        if n > remain {
            Err(())
        } else {
            unsafe { core::ptr::copy_nonoverlapping(sli.as_ptr(), self.cur, n) }
            self.cur = self.cur.wrapping_add(n);
            Ok(())
        }
    }

    #[inline]
    pub fn remain(&self) -> usize {
        (self.end as usize) - (self.cur as usize)
    }
}

impl<'a> From<&'a mut [u8]> for SerStream<'a> {
    fn from(value: &'a mut [u8]) -> Self {
        let len = value.len();
        let base: *mut u8 = value.as_mut_ptr();
        let end: *mut u8 = base.wrapping_add(len);
        SerStream {
            cur: base,
            end,
            _plt: PhantomData,
        }
    }
}

pub type SerFunc = unsafe fn(&mut SerStream, NonNull<()>) -> Result<(), ()>;
pub struct SerField {
    pub offset: usize,
    pub func: SerFunc,
}

/// # Safety
/// don't mess it up
pub trait Serialize
where
    Self: Sized,
{
    fn serialize(&self, stream: &mut SerStream) -> Result<(), ()>;
}

pub struct DeserStream<'a> {
    cur: *const u8,
    end: *const u8,
    _plt: PhantomData<&'a u8>,
}

impl<'a> DeserStream<'a> {
    pub fn pop_one(&mut self) -> Result<u8, ()> {
        if self.cur == self.end {
            Err(())
        } else {
            let val = unsafe { self.cur.read() };
            self.cur = self.cur.wrapping_add(1);
            Ok(val)
        }
    }

    pub fn pop_n(&mut self, n: usize) -> Result<&'a [u8], ()> {
        let remain = self.remain();
        if n > remain {
            Err(())
        } else {
            let sli = unsafe { core::slice::from_raw_parts(self.cur, n) };
            self.cur = self.cur.wrapping_add(n);
            Ok(sli)
        }
    }

    #[inline]
    pub fn remain(&self) -> usize {
        (self.end as usize) - (self.cur as usize)
    }
}

impl<'a> From<&'a [u8]> for DeserStream<'a> {
    fn from(value: &'a [u8]) -> Self {
        let len = value.len();
        let base = value.as_ptr();
        let end = base.wrapping_add(len);
        DeserStream {
            cur: base,
            end,
            _plt: PhantomData,
        }
    }
}

pub type DeserFunc = unsafe fn(&mut DeserStream, NonNull<()>) -> Result<(), ()>;
pub struct DeserField {
    pub offset: usize,
    pub func: DeserFunc,
}

/// # Safety
/// don't mess it up
pub trait Deserialize
where
    Self: Sized,
{
    fn deserialize(me: &mut MaybeUninit<Self>, stream: &mut DeserStream) -> Result<(), ()>;
}

pub mod impls {
    use core::{mem::size_of, ptr::addr_of_mut};

    use self::{
        de_varint::{
            de_zig_zag_i128, de_zig_zag_i16, de_zig_zag_i32, de_zig_zag_i64, try_take_varint_u128,
            try_take_varint_u16, try_take_varint_u32, try_take_varint_u64, try_take_varint_usize,
        },
        ser_varint::{
            varint_u128, varint_u16, varint_u32, varint_u64, varint_usize, zig_zag_i128,
            zig_zag_i16, zig_zag_i32, zig_zag_i64,
        },
    };

    use super::*;

    impl Serialize for () {
        #[inline]
        fn serialize(&self, _stream: &mut SerStream) -> Result<(), ()> {
            Ok(())
        }
    }

    impl Serialize for bool {
        #[inline]
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            let val: bool = *self;
            stream.push_one(if val { 0x01 } else { 0x00 })
        }
    }

    impl Serialize for u8 {
        #[inline]
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            let val: u8 = *self;
            stream.push_one(val)
        }
    }

    impl Serialize for u16 {
        #[inline]
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            let val: u16 = *self;
            varint_u16(val, stream)
        }
    }

    impl Serialize for u32 {
        #[inline]
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            let val: u32 = *self;
            varint_u32(val, stream)
        }
    }

    impl Serialize for u64 {
        #[inline]
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            let val: u64 = *self;
            varint_u64(val, stream)
        }
    }

    impl Serialize for u128 {
        #[inline]
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            let val: u128 = *self;
            varint_u128(val, stream)
        }
    }

    impl Serialize for usize {
        #[inline]
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            let val: usize = *self;
            varint_usize(val, stream)
        }
    }

    impl Serialize for f32 {
        #[inline]
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            let val: f32 = *self;
            let val = val.to_le_bytes();
            stream.push_n(&val)
        }
    }

    impl Serialize for f64 {
        #[inline]
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            let val: f64 = *self;
            let val = val.to_le_bytes();
            stream.push_n(&val)
        }
    }

    impl Serialize for i8 {
        #[inline]
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            let val: i8 = *self;
            stream.push_one(val as u8)
        }
    }

    impl Serialize for i16 {
        #[inline]
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            let val: i16 = *self;
            let val: u16 = zig_zag_i16(val);
            varint_u16(val, stream)
        }
    }

    impl Serialize for i32 {
        #[inline]
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            let val: i32 = *self;
            let val: u32 = zig_zag_i32(val);
            varint_u32(val, stream)
        }
    }

    impl Serialize for i64 {
        #[inline]
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            let val: i64 = *self;
            let val: u64 = zig_zag_i64(val);
            varint_u64(val, stream)
        }
    }

    impl Serialize for i128 {
        #[inline]
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            let val: i128 = *self;
            let val: u128 = zig_zag_i128(val);
            varint_u128(val, stream)
        }
    }

    impl Serialize for isize {
        #[inline]
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            let val: isize = *self;

            #[cfg(target_pointer_width = "16")]
            let val: usize = zig_zag_i16(val as i16) as usize;

            #[cfg(target_pointer_width = "32")]
            let val: usize = zig_zag_i32(val as i32) as usize;

            #[cfg(target_pointer_width = "64")]
            let val: usize = zig_zag_i64(val as i64) as usize;

            varint_usize(val, stream)
        }
    }

    #[cfg(feature = "std")]
    impl Serialize for String {
        #[inline]
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            let val: &String = self;
            let len = val.len();
            len.serialize(stream)?;
            let bytes = val.as_bytes();
            stream.push_n(bytes)
        }
    }

    #[cfg(feature = "std")]
    impl<T: Serialize> Serialize for Vec<T> {
        #[inline]
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            let len = self.len();
            len.serialize(stream)?;
            self.iter().try_for_each(|t| t.serialize(stream))
        }
    }

    impl<T: Serialize, const N: usize> Serialize for [T; N] {
        #[inline]
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            self.iter().try_for_each(|t| t.serialize(stream))
        }
    }

    impl<T: Serialize> Serialize for Option<T> {
        #[inline]
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            self.is_some().serialize(stream)?;
            if let Some(v) = self {
                v.serialize(stream)
            } else {
                Ok(())
            }
        }
    }

    impl<T: Serialize> Serialize for (T,) {
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            self.0.serialize(stream)
        }
    }

    impl<T: Serialize, U: Serialize> Serialize for (T, U) {
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            self.0.serialize(stream)?;
            self.1.serialize(stream)
        }
    }

    impl<T: Serialize, U: Serialize, V: Serialize> Serialize for (T, U, V) {
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            self.0.serialize(stream)?;
            self.1.serialize(stream)?;
            self.2.serialize(stream)
        }
    }

    impl<T: Serialize, U: Serialize, V: Serialize, W: Serialize> Serialize for (T, U, V, W) {
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            self.0.serialize(stream)?;
            self.1.serialize(stream)?;
            self.2.serialize(stream)?;
            self.3.serialize(stream)
        }
    }

    impl<T: Serialize, U: Serialize, V: Serialize, W: Serialize, X: Serialize> Serialize
        for (T, U, V, W, X)
    {
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            self.0.serialize(stream)?;
            self.1.serialize(stream)?;
            self.2.serialize(stream)?;
            self.3.serialize(stream)?;
            self.4.serialize(stream)
        }
    }

    impl<T: Serialize, U: Serialize, V: Serialize, W: Serialize, X: Serialize, Y: Serialize>
        Serialize for (T, U, V, W, X, Y)
    {
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            self.0.serialize(stream)?;
            self.1.serialize(stream)?;
            self.2.serialize(stream)?;
            self.3.serialize(stream)?;
            self.4.serialize(stream)?;
            self.5.serialize(stream)
        }
    }

    impl<
            T: Serialize,
            U: Serialize,
            V: Serialize,
            W: Serialize,
            X: Serialize,
            Y: Serialize,
            Z: Serialize,
        > Serialize for (T, U, V, W, X, Y, Z)
    {
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            self.0.serialize(stream)?;
            self.1.serialize(stream)?;
            self.2.serialize(stream)?;
            self.3.serialize(stream)?;
            self.4.serialize(stream)?;
            self.5.serialize(stream)?;
            self.6.serialize(stream)
        }
    }

    impl Deserialize for () {
        #[inline]
        fn deserialize(_me: &mut MaybeUninit<()>, _stream: &mut DeserStream) -> Result<(), ()> {
            Ok(())
        }
    }

    impl Deserialize for bool {
        #[inline]
        fn deserialize(me: &mut MaybeUninit<bool>, stream: &mut DeserStream) -> Result<(), ()> {
            match stream.pop_one() {
                Ok(0) => me.write(false),
                Ok(1) => me.write(true),
                _ => return Err(()),
            };
            Ok(())
        }
    }
    impl Deserialize for u8 {
        #[inline]
        fn deserialize(me: &mut MaybeUninit<u8>, stream: &mut DeserStream) -> Result<(), ()> {
            if let Ok(val) = stream.pop_one() {
                me.write(val);
                Ok(())
            } else {
                Err(())
            }
        }
    }
    impl Deserialize for u16 {
        #[inline]
        fn deserialize(me: &mut MaybeUninit<u16>, stream: &mut DeserStream) -> Result<(), ()> {
            if let Ok(val) = try_take_varint_u16(stream) {
                me.write(val);
                Ok(())
            } else {
                Err(())
            }
        }
    }
    impl Deserialize for u32 {
        #[inline]
        fn deserialize(me: &mut MaybeUninit<u32>, stream: &mut DeserStream) -> Result<(), ()> {
            if let Ok(val) = try_take_varint_u32(stream) {
                me.write(val);
                Ok(())
            } else {
                Err(())
            }
        }
    }
    impl Deserialize for u64 {
        #[inline]
        fn deserialize(me: &mut MaybeUninit<u64>, stream: &mut DeserStream) -> Result<(), ()> {
            if let Ok(val) = try_take_varint_u64(stream) {
                me.write(val);
                Ok(())
            } else {
                Err(())
            }
        }
    }
    impl Deserialize for u128 {
        #[inline]
        fn deserialize(me: &mut MaybeUninit<u128>, stream: &mut DeserStream) -> Result<(), ()> {
            if let Ok(val) = try_take_varint_u128(stream) {
                me.write(val);
                Ok(())
            } else {
                Err(())
            }
        }
    }
    impl Deserialize for usize {
        #[inline]
        fn deserialize(me: &mut MaybeUninit<usize>, stream: &mut DeserStream) -> Result<(), ()> {
            if let Ok(val) = try_take_varint_usize(stream) {
                me.write(val);
                Ok(())
            } else {
                Err(())
            }
        }
    }
    impl Deserialize for f32 {
        #[inline]
        fn deserialize(me: &mut MaybeUninit<f32>, stream: &mut DeserStream) -> Result<(), ()> {
            if let Ok(bytes) = stream.pop_n(size_of::<f32>()) {
                let mut buf = [0u8; size_of::<f32>()];
                buf.copy_from_slice(bytes);
                let val = f32::from_le_bytes(buf);
                me.write(val);
                Ok(())
            } else {
                Err(())
            }
        }
    }
    impl Deserialize for f64 {
        #[inline]
        fn deserialize(me: &mut MaybeUninit<f64>, stream: &mut DeserStream) -> Result<(), ()> {
            if let Ok(bytes) = stream.pop_n(size_of::<f64>()) {
                let mut buf = [0u8; size_of::<f64>()];
                buf.copy_from_slice(bytes);
                let val = f64::from_le_bytes(buf);
                me.write(val);
                Ok(())
            } else {
                Err(())
            }
        }
    }
    impl Deserialize for i8 {
        #[inline]
        fn deserialize(me: &mut MaybeUninit<i8>, stream: &mut DeserStream) -> Result<(), ()> {
            if let Ok(val) = stream.pop_one() {
                me.write(val as i8);
                Ok(())
            } else {
                Err(())
            }
        }
    }
    impl Deserialize for i16 {
        #[inline]
        fn deserialize(me: &mut MaybeUninit<i16>, stream: &mut DeserStream) -> Result<(), ()> {
            if let Ok(val) = try_take_varint_u16(stream) {
                let val = de_zig_zag_i16(val);
                me.write(val);
                Ok(())
            } else {
                Err(())
            }
        }
    }
    impl Deserialize for i32 {
        #[inline]
        fn deserialize(me: &mut MaybeUninit<i32>, stream: &mut DeserStream) -> Result<(), ()> {
            if let Ok(val) = try_take_varint_u32(stream) {
                let val = de_zig_zag_i32(val);
                me.write(val);
                Ok(())
            } else {
                Err(())
            }
        }
    }
    impl Deserialize for i64 {
        #[inline]
        fn deserialize(me: &mut MaybeUninit<i64>, stream: &mut DeserStream) -> Result<(), ()> {
            if let Ok(val) = try_take_varint_u64(stream) {
                let val = de_zig_zag_i64(val);
                me.write(val);
                Ok(())
            } else {
                Err(())
            }
        }
    }
    impl Deserialize for i128 {
        #[inline]
        fn deserialize(me: &mut MaybeUninit<i128>, stream: &mut DeserStream) -> Result<(), ()> {
            if let Ok(val) = try_take_varint_u128(stream) {
                let val = de_zig_zag_i128(val);
                me.write(val);
                Ok(())
            } else {
                Err(())
            }
        }
    }
    impl Deserialize for isize {
        #[inline]
        fn deserialize(me: &mut MaybeUninit<isize>, stream: &mut DeserStream) -> Result<(), ()> {
            if let Ok(val) = try_take_varint_usize(stream) {
                #[cfg(target_pointer_width = "16")]
                let val = de_zig_zag_i16(val as u16) as isize;
                #[cfg(target_pointer_width = "32")]
                let val = de_zig_zag_i32(val as u32) as isize;
                #[cfg(target_pointer_width = "64")]
                let val = de_zig_zag_i64(val as u64) as isize;
                me.write(val);
                Ok(())
            } else {
                Err(())
            }
        }
    }

    #[cfg(feature = "std")]
    impl Deserialize for String {
        #[inline]
        fn deserialize(me: &mut MaybeUninit<String>, stream: &mut DeserStream) -> Result<(), ()> {
            let mut len = MaybeUninit::<usize>::uninit();
            usize::deserialize(&mut len, stream)?;
            let len = unsafe { len.assume_init() };
            let bytes = stream.pop_n(len)?;
            let utf = core::str::from_utf8(bytes).map_err(drop)?;
            let s = utf.to_string();
            me.write(s);
            Ok(())
        }
    }

    #[cfg(feature = "std")]
    impl<T: Deserialize> Deserialize for Vec<T> {
        #[inline]
        fn deserialize(me: &mut MaybeUninit<Vec<T>>, stream: &mut DeserStream) -> Result<(), ()> {
            let mut len = MaybeUninit::<usize>::uninit();
            usize::deserialize(&mut len, stream)?;
            let len = unsafe { len.assume_init() };

            let mut out = Vec::<T>::with_capacity(len);
            let elems = out.spare_capacity_mut();
            elems
                .iter_mut()
                .take(len)
                .try_for_each(|t| T::deserialize(t, stream))?;

            unsafe {
                out.set_len(len);
            }
            me.write(out);
            Ok(())
        }
    }

    impl<T: Deserialize, const N: usize> Deserialize for [T; N] {
        #[inline]
        fn deserialize(me: &mut MaybeUninit<[T; N]>, stream: &mut DeserStream) -> Result<(), ()> {
            let sli = unsafe {
                core::slice::from_raw_parts_mut(me.as_mut_ptr().cast::<MaybeUninit<T>>(), N)
            };
            sli.iter_mut().try_for_each(|t| T::deserialize(t, stream))
        }
    }

    impl<T: Deserialize> Deserialize for Option<T> {
        #[inline]
        fn deserialize(
            me: &mut MaybeUninit<Option<T>>,
            stream: &mut DeserStream,
        ) -> Result<(), ()> {
            let mut disc = MaybeUninit::<bool>::uninit();
            bool::deserialize(&mut disc, stream)?;
            let disc = unsafe { disc.assume_init() };

            if disc {
                let mut out = MaybeUninit::<T>::uninit();
                T::deserialize(&mut out, stream)?;
                me.write(Some(unsafe { out.assume_init() }));
            } else {
                me.write(None);
            }

            Ok(())
        }
    }

    impl<T: Deserialize> Deserialize for (T,) {
        fn deserialize(me: &mut MaybeUninit<Self>, stream: &mut DeserStream) -> Result<(), ()> {
            T::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).0).cast::<MaybeUninit<T>>() },
                stream,
            )
        }
    }

    impl<T: Deserialize, U: Deserialize> Deserialize for (T, U) {
        fn deserialize(me: &mut MaybeUninit<Self>, stream: &mut DeserStream) -> Result<(), ()> {
            T::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).0).cast::<MaybeUninit<T>>() },
                stream,
            )?;
            U::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).1).cast::<MaybeUninit<U>>() },
                stream,
            )
        }
    }

    impl<T: Deserialize, U: Deserialize, V: Deserialize> Deserialize for (T, U, V) {
        fn deserialize(me: &mut MaybeUninit<Self>, stream: &mut DeserStream) -> Result<(), ()> {
            T::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).0).cast::<MaybeUninit<T>>() },
                stream,
            )?;
            U::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).1).cast::<MaybeUninit<U>>() },
                stream,
            )?;
            V::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).2).cast::<MaybeUninit<V>>() },
                stream,
            )
        }
    }

    impl<T: Deserialize, U: Deserialize, V: Deserialize, W: Deserialize> Deserialize for (T, U, V, W) {
        fn deserialize(me: &mut MaybeUninit<Self>, stream: &mut DeserStream) -> Result<(), ()> {
            T::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).0).cast::<MaybeUninit<T>>() },
                stream,
            )?;
            U::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).1).cast::<MaybeUninit<U>>() },
                stream,
            )?;
            V::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).2).cast::<MaybeUninit<V>>() },
                stream,
            )?;
            W::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).3).cast::<MaybeUninit<W>>() },
                stream,
            )
        }
    }

    impl<T: Deserialize, U: Deserialize, V: Deserialize, W: Deserialize, X: Deserialize> Deserialize
        for (T, U, V, W, X)
    {
        fn deserialize(me: &mut MaybeUninit<Self>, stream: &mut DeserStream) -> Result<(), ()> {
            T::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).0).cast::<MaybeUninit<T>>() },
                stream,
            )?;
            U::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).1).cast::<MaybeUninit<U>>() },
                stream,
            )?;
            V::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).2).cast::<MaybeUninit<V>>() },
                stream,
            )?;
            W::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).3).cast::<MaybeUninit<W>>() },
                stream,
            )?;
            X::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).4).cast::<MaybeUninit<X>>() },
                stream,
            )
        }
    }

    impl<
            T: Deserialize,
            U: Deserialize,
            V: Deserialize,
            W: Deserialize,
            X: Deserialize,
            Y: Deserialize,
        > Deserialize for (T, U, V, W, X, Y)
    {
        fn deserialize(me: &mut MaybeUninit<Self>, stream: &mut DeserStream) -> Result<(), ()> {
            T::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).0).cast::<MaybeUninit<T>>() },
                stream,
            )?;
            U::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).1).cast::<MaybeUninit<U>>() },
                stream,
            )?;
            V::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).2).cast::<MaybeUninit<V>>() },
                stream,
            )?;
            W::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).3).cast::<MaybeUninit<W>>() },
                stream,
            )?;
            X::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).4).cast::<MaybeUninit<X>>() },
                stream,
            )?;
            Y::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).5).cast::<MaybeUninit<Y>>() },
                stream,
            )
        }
    }

    impl<
            T: Deserialize,
            U: Deserialize,
            V: Deserialize,
            W: Deserialize,
            X: Deserialize,
            Y: Deserialize,
            Z: Deserialize,
        > Deserialize for (T, U, V, W, X, Y, Z)
    {
        fn deserialize(me: &mut MaybeUninit<Self>, stream: &mut DeserStream) -> Result<(), ()> {
            T::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).0).cast::<MaybeUninit<T>>() },
                stream,
            )?;
            U::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).1).cast::<MaybeUninit<U>>() },
                stream,
            )?;
            V::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).2).cast::<MaybeUninit<V>>() },
                stream,
            )?;
            W::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).3).cast::<MaybeUninit<W>>() },
                stream,
            )?;
            X::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).4).cast::<MaybeUninit<X>>() },
                stream,
            )?;
            Y::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).5).cast::<MaybeUninit<Y>>() },
                stream,
            )?;
            Z::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).6).cast::<MaybeUninit<Z>>() },
                stream,
            )
        }
    }
}

pub(crate) mod ser_varint {
    // copy and paste from postcard

    use crate::SerStream;

    /// Returns the maximum number of bytes required to encode T.
    pub const fn varint_max<T: Sized>() -> usize {
        const BITS_PER_BYTE: usize = 8;
        const BITS_PER_VARINT_BYTE: usize = 7;

        // How many data bits do we need for this type?
        let bits = core::mem::size_of::<T>() * BITS_PER_BYTE;

        // We add (BITS_PER_VARINT_BYTE - 1), to ensure any integer divisions
        // with a remainder will always add exactly one full byte, but
        // an evenly divided number of bits will be the same
        let roundup_bits = bits + (BITS_PER_VARINT_BYTE - 1);

        // Apply division, using normal "round down" integer division
        roundup_bits / BITS_PER_VARINT_BYTE
    }

    #[inline]
    pub fn varint_usize(mut value: usize, out: &mut SerStream) -> Result<(), ()> {
        loop {
            let now = value.to_le_bytes()[0];
            if value < 128 {
                return out.push_one(now);
            } else {
                out.push_one(now | 0x80)?;
            }

            value >>= 7;
        }
    }

    #[inline]
    pub fn varint_u16(mut value: u16, out: &mut SerStream) -> Result<(), ()> {
        loop {
            let now = value.to_le_bytes()[0];
            if value < 128 {
                return out.push_one(now);
            } else {
                out.push_one(now | 0x80)?;
            }

            value >>= 7;
        }
    }

    #[inline]
    pub fn varint_u32(mut value: u32, out: &mut SerStream) -> Result<(), ()> {
        loop {
            let now = value.to_le_bytes()[0];
            if value < 128 {
                return out.push_one(now);
            } else {
                out.push_one(now | 0x80)?;
            }

            value >>= 7;
        }
    }

    #[inline]
    pub fn varint_u64(mut value: u64, out: &mut SerStream) -> Result<(), ()> {
        loop {
            let now = value.to_le_bytes()[0];
            if value < 128 {
                return out.push_one(now);
            } else {
                out.push_one(now | 0x80)?;
            }

            value >>= 7;
        }
    }

    #[inline]
    pub fn varint_u128(mut value: u128, out: &mut SerStream) -> Result<(), ()> {
        loop {
            let now = value.to_le_bytes()[0];
            if value < 128 {
                return out.push_one(now);
            } else {
                out.push_one(now | 0x80)?;
            }

            value >>= 7;
        }
    }

    pub fn zig_zag_i16(n: i16) -> u16 {
        ((n << 1) ^ (n >> 15)) as u16
    }

    pub fn zig_zag_i32(n: i32) -> u32 {
        ((n << 1) ^ (n >> 31)) as u32
    }

    pub fn zig_zag_i64(n: i64) -> u64 {
        ((n << 1) ^ (n >> 63)) as u64
    }

    pub fn zig_zag_i128(n: i128) -> u128 {
        ((n << 1) ^ (n >> 127)) as u128
    }
}

mod de_varint {
    // copy and paste from postcard

    use crate::{ser_varint::varint_max, DeserStream};

    /// Returns the maximum value stored in the last encoded byte.
    pub const fn max_of_last_byte<T: Sized>() -> u8 {
        let max_bits = core::mem::size_of::<T>() * 8;
        let extra_bits = max_bits % 7;
        (1 << extra_bits) - 1
    }

    pub fn de_zig_zag_i16(n: u16) -> i16 {
        ((n >> 1) as i16) ^ (-((n & 0b1) as i16))
    }

    pub fn de_zig_zag_i32(n: u32) -> i32 {
        ((n >> 1) as i32) ^ (-((n & 0b1) as i32))
    }

    pub fn de_zig_zag_i64(n: u64) -> i64 {
        ((n >> 1) as i64) ^ (-((n & 0b1) as i64))
    }

    pub fn de_zig_zag_i128(n: u128) -> i128 {
        ((n >> 1) as i128) ^ (-((n & 0b1) as i128))
    }

    #[cfg(target_pointer_width = "16")]
    #[inline(always)]
    pub fn try_take_varint_usize(data: &mut DeserStream) -> Result<usize, ()> {
        try_take_varint_u16(data).map(|u| u as usize)
    }

    #[cfg(target_pointer_width = "32")]
    #[inline(always)]
    pub fn try_take_varint_usize(data: &mut DeserStream) -> Result<usize, ()> {
        try_take_varint_u32(data).map(|u| u as usize)
    }

    #[cfg(target_pointer_width = "64")]
    #[inline(always)]
    pub fn try_take_varint_usize(data: &mut DeserStream) -> Result<usize, ()> {
        try_take_varint_u64(data).map(|u| u as usize)
    }

    #[inline]
    pub fn try_take_varint_u16(data: &mut DeserStream) -> Result<u16, ()> {
        let mut out = 0;
        for i in 0..varint_max::<u16>() {
            let val = data.pop_one()?;
            let carry = (val & 0x7F) as u16;
            out |= carry << (7 * i);

            if (val & 0x80) == 0 {
                if i == varint_max::<u16>() - 1 && val > max_of_last_byte::<u16>() {
                    return Err(());
                } else {
                    return Ok(out);
                }
            }
        }
        Err(())
    }

    #[inline]
    pub fn try_take_varint_u32(data: &mut DeserStream) -> Result<u32, ()> {
        let mut out = 0;
        for i in 0..varint_max::<u32>() {
            let val = data.pop_one()?;
            let carry = (val & 0x7F) as u32;
            out |= carry << (7 * i);

            if (val & 0x80) == 0 {
                if i == varint_max::<u32>() - 1 && val > max_of_last_byte::<u32>() {
                    return Err(());
                } else {
                    return Ok(out);
                }
            }
        }
        Err(())
    }

    #[inline]
    pub fn try_take_varint_u64(data: &mut DeserStream) -> Result<u64, ()> {
        let mut out = 0;
        for i in 0..varint_max::<u64>() {
            let val = data.pop_one()?;
            let carry = (val & 0x7F) as u64;
            out |= carry << (7 * i);

            if (val & 0x80) == 0 {
                if i == varint_max::<u64>() - 1 && val > max_of_last_byte::<u64>() {
                    return Err(());
                } else {
                    return Ok(out);
                }
            }
        }
        Err(())
    }

    #[inline]
    pub fn try_take_varint_u128(data: &mut DeserStream) -> Result<u128, ()> {
        let mut out = 0;
        for i in 0..varint_max::<u128>() {
            let val = data.pop_one()?;
            let carry = (val & 0x7F) as u128;
            out |= carry << (7 * i);

            if (val & 0x80) == 0 {
                if i == varint_max::<u128>() - 1 && val > max_of_last_byte::<u128>() {
                    return Err(());
                } else {
                    return Ok(out);
                }
            }
        }
        Err(())
    }
}

#[cfg(all(test, feature = "std"))]
mod test {
    use super::*;
    use core::{mem::offset_of, ptr::addr_of_mut};

    #[derive(Debug, PartialEq)]
    struct Alpha {
        a: u8,
        b: u16,
        c: u32,
        d: i8,
        e: i16,
        f: i32,
    }

    #[derive(Debug, PartialEq)]
    struct Beta {
        a: u8,
        b: u16,
        c: u32,
        d: i8,
        e: i16,
        f: i32,
    }

    // #[derive(Debug, PartialEq)]
    // enum Dolsot {
    //     Bib(Alpha),
    //     Bim(Beta),
    //     Bap(u32),
    //     Bowl,
    // }

    // // THESE ARE THE PARTS THAT WILL HAVE TO BE MACRO GENERATED
    // //
    // //

    // #[inline]
    // pub unsafe fn ser_dolsot(stream: &mut SerStream, base: NonNull<()>) -> Result<(), ()> {
    //     let eref = base.cast::<Dolsot>().as_ref();
    //     let (var, fun, valref): (u32, SerFunc, NonNull<()>) = match eref {
    //         Dolsot::Bib(x) => (0, ser_fields::<Alpha>, NonNull::from(x).cast::<()>()),
    //         Dolsot::Bim(x) => (1, ser_fields::<Beta>, NonNull::from(x).cast::<()>()),
    //         Dolsot::Bap(x) => (2, impls::ser_u32, NonNull::from(x).cast::<()>()),
    //         Dolsot::Bowl => (3, impls::ser_nothing, NonNull::<()>::dangling()),
    //     };

    //     // serialize the discriminant as a u32
    //     if impls::ser_u32(stream, NonNull::from(&var).cast()).is_err() {
    //         return Err(());
    //     }

    //     // Serialize the payload
    //     (fun)(stream, valref)
    // }

    // #[inline]
    // pub unsafe fn deser_dolsot(stream: &mut DeserStream, base: NonNull<()>) -> Result<(), ()> {
    //     let mut disc = MaybeUninit::<u32>::uninit();
    //     let dolsot_ref = base.cast::<Dolsot>();
    //     if impls::deser_u32(stream, NonNull::from(&mut disc).cast()).is_err() {
    //         return Err(());
    //     }
    //     let disc = disc.assume_init();
    //     match disc {
    //         0 => {
    //             // Bib
    //             let mut val = MaybeUninit::<Alpha>::uninit();
    //             deser_fields::<Alpha>(stream, NonNull::from(&mut val).cast())?;
    //             dolsot_ref.as_ptr().write(Dolsot::Bib(val.assume_init()));
    //         }
    //         1 => {
    //             // Bim
    //             let mut val = MaybeUninit::<Beta>::uninit();
    //             deser_fields::<Beta>(stream, NonNull::from(&mut val).cast())?;
    //             dolsot_ref.as_ptr().write(Dolsot::Bim(val.assume_init()));
    //         }
    //         2 => {
    //             // Bap
    //             let mut val = MaybeUninit::<u32>::uninit();
    //             impls::deser_u32(stream, NonNull::from(&mut val).cast())?;
    //             dolsot_ref.as_ptr().write(Dolsot::Bap(val.assume_init()));
    //         }
    //         3 => dolsot_ref.as_ptr().write(Dolsot::Bowl),
    //         _ => return Err(()),
    //     }
    //     Ok(())
    // }

    impl Serialize for Alpha {
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            self.a.serialize(stream)?;
            self.b.serialize(stream)?;
            self.c.serialize(stream)?;
            self.d.serialize(stream)?;
            self.e.serialize(stream)?;
            self.f.serialize(stream)
        }
    }

    impl Serialize for Beta {
        fn serialize(&self, stream: &mut SerStream) -> Result<(), ()> {
            self.a.serialize(stream)?;
            self.b.serialize(stream)?;
            self.c.serialize(stream)?;
            self.d.serialize(stream)?;
            self.e.serialize(stream)?;
            self.f.serialize(stream)
        }
    }

    impl Deserialize for Alpha {
        fn deserialize(me: &mut MaybeUninit<Alpha>, stream: &mut DeserStream) -> Result<(), ()> {
            u8::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).a).cast::<MaybeUninit<u8>>() },
                stream,
            )?;
            u16::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).b).cast::<MaybeUninit<u16>>() },
                stream,
            )?;
            u32::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).c).cast::<MaybeUninit<u32>>() },
                stream,
            )?;
            i8::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).d).cast::<MaybeUninit<i8>>() },
                stream,
            )?;
            i16::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).e).cast::<MaybeUninit<i16>>() },
                stream,
            )?;
            i32::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).f).cast::<MaybeUninit<i32>>() },
                stream,
            )
        }
    }

    impl Deserialize for Beta {
        fn deserialize(me: &mut MaybeUninit<Beta>, stream: &mut DeserStream) -> Result<(), ()> {
            u8::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).a).cast::<MaybeUninit<u8>>() },
                stream,
            )?;
            u16::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).b).cast::<MaybeUninit<u16>>() },
                stream,
            )?;
            u32::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).c).cast::<MaybeUninit<u32>>() },
                stream,
            )?;
            i8::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).d).cast::<MaybeUninit<i8>>() },
                stream,
            )?;
            i16::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).e).cast::<MaybeUninit<i16>>() },
                stream,
            )?;
            i32::deserialize(
                unsafe { &mut *addr_of_mut!((*me.as_mut_ptr()).f).cast::<MaybeUninit<i32>>() },
                stream,
            )
        }
    }

    // unsafe impl Serialize for Dolsot {
    //     const FIELDS: &'static [SerField] = &[SerField {
    //         offset: 0,
    //         func: ser_dolsot,
    //     }];
    // }

    // unsafe impl Deserialize for Dolsot {
    //     const FIELDS: &'static [DeserField] = &[DeserField {
    //         offset: 0,
    //         func: deser_dolsot,
    //     }];
    // }

    // //
    // //
    // // END OF MACRO GENERATION

    // #[test]
    // fn smoke_enum() {
    //     let a = Dolsot::Bim(Beta {
    //         a: 1,
    //         b: 256,
    //         c: 65536,
    //         d: -1,
    //         e: -129,
    //         f: -32769,
    //     });

    //     let mut outa = [0u8; 64];
    //     let mut sers = SerStream::from(outa.as_mut_slice());
    //     unsafe {
    //         ser_fields_ref(&mut sers, &a).unwrap();
    //     }
    //     let remain = sers.remain();
    //     let used = outa.len() - remain;
    //     assert_eq!(used, 13);
    //     assert_eq!(
    //         &outa[..used],
    //         &[1, 1, 128, 2, 128, 128, 4, 255, 129, 2, 129, 128, 4]
    //     );

    //     // -

    //     let mut desers = DeserStream::from(&outa[..used]);
    //     let mut out = MaybeUninit::<Dolsot>::uninit();
    //     unsafe {
    //         deser_fields_ref(&mut desers, &mut out).unwrap();
    //     }
    //     let remain = desers.remain();
    //     assert_eq!(remain, 0);
    //     let out = unsafe { out.assume_init() };
    //     assert_eq!(
    //         Dolsot::Bim(Beta {
    //             a: 1,
    //             b: 256,
    //             c: 65536,
    //             d: -1,
    //             e: -129,
    //             f: -32769,
    //         }),
    //         out,
    //     );
    // }

    #[test]
    fn smoke_ser() {
        let a = Alpha {
            a: 1,
            b: 256,
            c: 65536,
            d: -1,
            e: -129,
            f: -32769,
        };

        let mut outa = [0u8; 64];
        let mut sers = SerStream::from(outa.as_mut_slice());
        a.serialize(&mut sers).unwrap();
        let remain = sers.remain();
        let used = outa.len() - remain;
        assert_eq!(used, 12);
        assert_eq!(
            &outa[..used],
            &[1, 128, 2, 128, 128, 4, 255, 129, 2, 129, 128, 4]
        );

        let b = Beta {
            a: 1,
            b: 256,
            c: 65536,
            d: -1,
            e: -129,
            f: -32769,
        };

        let mut outb = [0u8; 64];
        let mut sers = SerStream::from(outb.as_mut_slice());
        b.serialize(&mut sers).unwrap();
        let remain = sers.remain();
        let used = outb.len() - remain;
        assert_eq!(used, 12);
        assert_eq!(
            &outb[..used],
            &[1, 128, 2, 128, 128, 4, 255, 129, 2, 129, 128, 4]
        );
    }

    #[test]
    fn smoke_deser() {
        let bytes = &[1, 128, 2, 128, 128, 4, 255, 129, 2, 129, 128, 4];

        let mut desers = DeserStream::from(bytes.as_slice());
        let mut out = MaybeUninit::<Alpha>::uninit();
        Alpha::deserialize(&mut out, &mut desers).unwrap();
        let remain = desers.remain();
        assert_eq!(remain, 0);
        let out = unsafe { out.assume_init() };
        assert_eq!(
            Alpha {
                a: 1,
                b: 256,
                c: 65536,
                d: -1,
                e: -129,
                f: -32769,
            },
            out,
        );

        let mut desers = DeserStream::from(bytes.as_slice());
        let mut out = MaybeUninit::<Beta>::uninit();
        Beta::deserialize(&mut out, &mut desers).unwrap();
        let remain = desers.remain();
        assert_eq!(remain, 0);
        let out = unsafe { out.assume_init() };
        assert_eq!(
            Beta {
                a: 1,
                b: 256,
                c: 65536,
                d: -1,
                e: -129,
                f: -32769,
            },
            out,
        );
    }
}
