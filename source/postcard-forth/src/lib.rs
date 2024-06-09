#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![allow(clippy::result_unit_err, clippy::missing_safety_doc)]

use core::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

pub struct Alpha {
    pub a: u8,
    pub b: u16,
    pub c: u32,
}

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

/// # Safety
/// don't mess it up
pub unsafe trait Serialize {
    const NODE: SerNode;
}

type SerFunc = unsafe fn(&mut SerStream, NonNull<()>) -> Result<(), ()>;
pub struct SerField {
    pub offset: usize,
    pub node: &'static SerNode,
}

pub struct SerNode {
    pub len: usize,
    pub either: SerEither,
}

pub union SerEither {
    pub func: SerFunc,
    pub arry: NonNull<SerField>,
}

impl SerNode {
    pub const fn new_custom(func: SerFunc) -> Self {
        SerNode {
            len: 0,
            either: SerEither { func }
        }
    }

    pub const fn new_arry(arr: &[SerField]) -> Self {
        assert!(!arr.is_empty());
        let len = arr.len();
        let arry = unsafe { NonNull::new_unchecked(arr.as_ptr().cast_mut()) };
        SerNode {
            len,
            either: SerEither { arry },
        }
    }
}

pub unsafe fn ser_node(
    stream: &mut SerStream,
    base: NonNull<()>,
    node: &'static SerNode,
) -> Result<(), ()> {
    if node.len == 0 {
        (node.either.func)(stream, base)
    } else {
        let arr = core::slice::from_raw_parts(node.either.arry.as_ptr(), node.len);
        ser_node_arry(stream, base, arr)
    }?;
    Ok(())
}

unsafe fn ser_node_arry(
    stream: &mut SerStream,
    base: NonNull<()>,
    items: &'static [SerField],
) -> Result<(), ()> {
    for item in items {
        let item_base = NonNull::new_unchecked(base.as_ptr().byte_add(item.offset));
        ser_node(stream, item_base, item.node)?;
    }
    Ok(())
}

/// # Safety
/// don't mess it up
#[inline]
pub unsafe fn ser_fields_ref<S: Serialize>(stream: &mut SerStream, base: &S) -> Result<(), ()> {
    let nn_ptr: NonNull<S> = NonNull::from(base);
    let nn_erased: NonNull<()> = nn_ptr.cast();
    ser_node(stream, nn_erased, &S::NODE)?;
    Ok(())
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
    pub node: &'static DeserNode,
}

pub struct DeserNode {
    pub len: usize,
    pub either: DeserEither,
}

pub union DeserEither {
    pub func: DeserFunc,
    pub arry: NonNull<DeserField>,
}

impl DeserNode {
    pub const fn new_custom(func: DeserFunc) -> Self {
        DeserNode {
            len: 0,
            either: DeserEither { func }
        }
    }

    pub const fn new_arry(arr: &[DeserField]) -> Self {
        assert!(!arr.is_empty());
        let len = arr.len();
        let arry = unsafe { NonNull::new_unchecked(arr.as_ptr().cast_mut()) };
        DeserNode {
            len,
            either: DeserEither { arry },
        }
    }
}

/// # Safety
/// don't mess it up
pub unsafe trait Deserialize {
    const NODE: DeserNode;
}

pub unsafe fn deser_node(
    stream: &mut DeserStream,
    base: NonNull<()>,
    node: &'static DeserNode,
) -> Result<(), ()> {
    if node.len == 0 {
        (node.either.func)(stream, base)
    } else {
        let arr = core::slice::from_raw_parts(node.either.arry.as_ptr(), node.len);
        deser_node_arry(stream, base, arr)
    }?;
    Ok(())
}

unsafe fn deser_node_arry(
    stream: &mut DeserStream,
    base: NonNull<()>,
    items: &'static [DeserField],
) -> Result<(), ()> {
    for item in items {
        let item_base = NonNull::new_unchecked(base.as_ptr().byte_add(item.offset));
        deser_node(stream, item_base, item.node)?;
    }
    Ok(())
}

/// # Safety
/// don't mess it up
#[inline]
pub unsafe fn deser_fields_ref<D: Deserialize>(
    stream: &mut DeserStream,
    base: &mut MaybeUninit<D>,
) -> Result<(), ()> {
    let nn_ptr: NonNull<MaybeUninit<D>> = NonNull::from(base);
    let nn_erased: NonNull<()> = nn_ptr.cast();
    deser_node(stream, nn_erased, &D::NODE)
}

pub mod impls {
    use core::mem::size_of;

    use self::{
        de_varint::{
            de_zig_zag_i128, de_zig_zag_i16, de_zig_zag_i32, de_zig_zag_i64, try_take_varint_u128,
            try_take_varint_u16, try_take_varint_u32, try_take_varint_u64, try_take_varint_usize,
        },
        ser_varint::{
            varint_u128, varint_u16, varint_u32, varint_u64, varint_usize,
            zig_zag_i128, zig_zag_i16, zig_zag_i32, zig_zag_i64,
        },
    };

    use super::*;

    #[inline]
    pub unsafe fn ser_nothing(_stream: &mut SerStream, _base: NonNull<()>) -> Result<(), ()> {
        Ok(())
    }

    #[inline]
    pub unsafe fn ser_bool(stream: &mut SerStream, base: NonNull<()>) -> Result<(), ()> {
        let val: bool = base.cast::<bool>().as_ptr().read();
        stream.push_one(if val { 0x01 } else { 0x00 })
    }

    #[inline]
    pub unsafe fn ser_u8(stream: &mut SerStream, base: NonNull<()>) -> Result<(), ()> {
        let val: u8 = base.cast::<u8>().as_ptr().read();
        stream.push_one(val)
    }

    #[inline]
    pub unsafe fn ser_u16(stream: &mut SerStream, base: NonNull<()>) -> Result<(), ()> {
        let val: u16 = base.cast::<u16>().as_ptr().read();
        varint_u16(val, stream)
    }

    #[inline]
    pub unsafe fn ser_u32(stream: &mut SerStream, base: NonNull<()>) -> Result<(), ()> {
        let val: u32 = base.cast::<u32>().as_ptr().read();
        varint_u32(val, stream)
    }

    #[inline]
    pub unsafe fn ser_u64(stream: &mut SerStream, base: NonNull<()>) -> Result<(), ()> {
        let val: u64 = base.cast::<u64>().as_ptr().read();
        varint_u64(val, stream)
    }

    #[inline]
    pub unsafe fn ser_u128(stream: &mut SerStream, base: NonNull<()>) -> Result<(), ()> {
        let val: u128 = base.cast::<u128>().as_ptr().read();
        varint_u128(val, stream)
    }

    #[inline]
    pub unsafe fn ser_usize(stream: &mut SerStream, base: NonNull<()>) -> Result<(), ()> {
        let val: usize = base.cast::<usize>().as_ptr().read();
        varint_usize(val, stream)
    }

    #[inline]
    pub unsafe fn ser_f32(stream: &mut SerStream, base: NonNull<()>) -> Result<(), ()> {
        let val: f32 = base.cast::<f32>().as_ptr().read();
        let val = val.to_le_bytes();
        stream.push_n(&val)
    }

    #[inline]
    pub unsafe fn ser_f64(stream: &mut SerStream, base: NonNull<()>) -> Result<(), ()> {
        let val: f64 = base.cast::<f64>().as_ptr().read();
        let val = val.to_le_bytes();
        stream.push_n(&val)
    }

    #[inline]
    pub unsafe fn ser_i8(stream: &mut SerStream, base: NonNull<()>) -> Result<(), ()> {
        let val: i8 = base.cast::<i8>().as_ptr().read();
        stream.push_one(val as u8)
    }

    #[inline]
    pub unsafe fn ser_i16(stream: &mut SerStream, base: NonNull<()>) -> Result<(), ()> {
        let val: i16 = base.cast::<i16>().as_ptr().read();
        let val: u16 = zig_zag_i16(val);
        varint_u16(val, stream)
    }

    #[inline]
    pub unsafe fn ser_i32(stream: &mut SerStream, base: NonNull<()>) -> Result<(), ()> {
        let val: i32 = base.cast::<i32>().as_ptr().read();
        let val: u32 = zig_zag_i32(val);
        varint_u32(val, stream)
    }

    #[inline]
    pub unsafe fn ser_i64(stream: &mut SerStream, base: NonNull<()>) -> Result<(), ()> {
        let val: i64 = base.cast::<i64>().as_ptr().read();
        let val: u64 = zig_zag_i64(val);
        varint_u64(val, stream)
    }

    #[inline]
    pub unsafe fn ser_i128(stream: &mut SerStream, base: NonNull<()>) -> Result<(), ()> {
        let val: i128 = base.cast::<i128>().as_ptr().read();
        let val: u128 = zig_zag_i128(val);
        varint_u128(val, stream)
    }

    #[inline]
    pub unsafe fn ser_isize(stream: &mut SerStream, base: NonNull<()>) -> Result<(), ()> {
        let val: isize = base.cast::<isize>().as_ptr().read();

        #[cfg(target_pointer_width = "16")]
        let val: usize = zig_zag_i16(val as i16) as usize;

        #[cfg(target_pointer_width = "32")]
        let val: usize = zig_zag_i32(val as i32) as usize;

        #[cfg(target_pointer_width = "64")]
        let val: usize = zig_zag_i64(val as i64) as usize;

        varint_usize(val, stream)
    }

    #[cfg(feature = "std")]
    #[inline]
    pub unsafe fn ser_string(stream: &mut SerStream, base: NonNull<()>) -> Result<(), ()> {
        let val: &String = base.cast::<String>().as_ref();
        let len = val.len();
        ser_usize(stream, NonNull::from(&len).cast())?;
        let bytes = val.as_bytes();
        stream.push_n(bytes)
    }

    #[cfg(feature = "std")]
    #[inline]
    pub unsafe fn ser_vec<T: Serialize>(
        stream: &mut SerStream,
        base: NonNull<()>,
    ) -> Result<(), ()> {
        let val: &Vec<T> = base.cast::<Vec<T>>().as_ref();
        let len = val.len();
        ser_usize(stream, NonNull::from(&len).cast())?;
        for t in val.iter() {
            ser_node(stream, NonNull::from(t).cast(), &T::NODE)?;
        }
        Ok(())
    }

    #[inline]
    pub unsafe fn ser_arr<T: Serialize, const N: usize>(
        stream: &mut SerStream,
        base: NonNull<()>,
    ) -> Result<(), ()> {
        let val: &[T; N] = base.cast::<[T; N]>().as_ref();
        for t in val.iter() {
            ser_node(stream, NonNull::from(t).cast(), &T::NODE)?;
        }
        Ok(())
    }

    #[inline]
    pub unsafe fn ser_option<T: Serialize>(
        stream: &mut SerStream,
        base: NonNull<()>,
    ) -> Result<(), ()> {
        let val: &Option<T> = base.cast::<Option<T>>().as_ref();
        let disc = val.is_some();
        ser_bool(stream, NonNull::from(&disc).cast())?;
        if let Some(v) = val {
            ser_node(stream, NonNull::from(v).cast(), &T::NODE)
        } else {
            Ok(())
        }
    }

    unsafe impl Serialize for bool {
        const NODE: SerNode = SerNode::new_custom(ser_bool);
    }

    unsafe impl Serialize for u8 {
        const NODE: SerNode = SerNode::new_custom(ser_u8);
    }

    unsafe impl Serialize for u16 {
        const NODE: SerNode = SerNode::new_custom(ser_u16);
    }

    unsafe impl Serialize for u32 {
        const NODE: SerNode = SerNode::new_custom(ser_u32);
    }

    unsafe impl Serialize for u64 {
        const NODE: SerNode = SerNode::new_custom(ser_u64);
    }

    unsafe impl Serialize for u128 {
        const NODE: SerNode = SerNode::new_custom(ser_u128);
    }

    unsafe impl Serialize for usize {
        const NODE: SerNode = SerNode::new_custom(ser_usize);
    }

    unsafe impl Serialize for f32 {
        const NODE: SerNode = SerNode::new_custom(ser_f32);
    }

    unsafe impl Serialize for f64 {
        const NODE: SerNode = SerNode::new_custom(ser_f64);
    }

    unsafe impl Serialize for i8 {
        const NODE: SerNode = SerNode::new_custom(ser_i8);
    }

    unsafe impl Serialize for i16 {
        const NODE: SerNode = SerNode::new_custom(ser_i16);
    }

    unsafe impl Serialize for i32 {
        const NODE: SerNode = SerNode::new_custom(ser_i32);
    }

    unsafe impl Serialize for i64 {
        const NODE: SerNode = SerNode::new_custom(ser_i64);
    }

    unsafe impl Serialize for i128 {
        const NODE: SerNode = SerNode::new_custom(ser_i128);
    }

    unsafe impl Serialize for isize {
        const NODE: SerNode = SerNode::new_custom(ser_isize);
    }

    #[cfg(feature = "std")]
    unsafe impl Serialize for String {
        const NODE: SerNode = SerNode::new_custom(ser_string);
    }

    #[cfg(feature = "std")]
    unsafe impl<T: Serialize> Serialize for Vec<T> {
        const NODE: SerNode = SerNode::new_custom(ser_vec::<T>);
    }

    unsafe impl<T: Serialize, const N: usize> Serialize for [T; N] {
        const NODE: SerNode = SerNode::new_custom(ser_arr::<T, N>);
    }

    unsafe impl<T: Serialize> Serialize for Option<T> {
        const NODE: SerNode = SerNode::new_custom(ser_option::<T>);
    }

    trait TupExtSer {
        const SER_FIELDS: &'static [SerField];
    }

    trait TupExtDeser {
        const DESER_FIELDS: &'static [DeserField];
    }

    unsafe impl<T: Serialize> Serialize for (T,) {
        const NODE: SerNode = SerNode::new_arry(Self::SER_FIELDS);
    }
    impl<T: Serialize> TupExtSer for (T,) {
        const SER_FIELDS: &'static [SerField] = &[SerField {
            offset: core::mem::offset_of!((T,), 0),
            node: &T::NODE,
        }];
    }

    unsafe impl<T: Serialize, U: Serialize> Serialize for (T, U) {
        const NODE: SerNode = SerNode::new_arry(Self::SER_FIELDS);
    }
    impl<T: Serialize, U: Serialize> TupExtSer for (T, U) {
        const SER_FIELDS: &'static [SerField] = &[
            SerField {
                offset: core::mem::offset_of!((T, U), 0),
                node: &T::NODE,
            },
            SerField {
                offset: core::mem::offset_of!((T, U), 1),
                node: &U::NODE
            },
        ];
    }

    unsafe impl<T: Serialize, U: Serialize, V: Serialize> Serialize for (T, U, V) {
        const NODE: SerNode = SerNode::new_arry(Self::SER_FIELDS);
    }
    impl<T: Serialize, U: Serialize, V: Serialize> TupExtSer for (T, U, V) {
        const SER_FIELDS: &'static [SerField] = &[
            SerField {
                offset: core::mem::offset_of!((T, U, V), 0),
                node: &T::NODE,
            },
            SerField {
                offset: core::mem::offset_of!((T, U, V), 1),
                node: &U::NODE
            },
            SerField {
                offset: core::mem::offset_of!((T, U, V), 2),
                node: &V::NODE,
            },
        ];
    }

    unsafe impl<T: Serialize, U: Serialize, V: Serialize, W: Serialize> Serialize for (T, U, V, W) {
        const NODE: SerNode = SerNode::new_arry(Self::SER_FIELDS);
    }
    impl<T: Serialize, U: Serialize, V: Serialize, W: Serialize> TupExtSer for (T, U, V, W) {
        const SER_FIELDS: &'static [SerField] = &[
            SerField {
                offset: core::mem::offset_of!((T, U, V, W), 0),
                node: &T::NODE,
            },
            SerField {
                offset: core::mem::offset_of!((T, U, V, W), 1),
                node: &U::NODE
            },
            SerField {
                offset: core::mem::offset_of!((T, U, V, W), 2),
                node: &V::NODE,
            },
            SerField {
                offset: core::mem::offset_of!((T, U, V, W), 3),
                node: &W::NODE,
            },
        ];
    }

    unsafe impl<T: Serialize, U: Serialize, V: Serialize, W: Serialize, X: Serialize> Serialize
        for (T, U, V, W, X)
    {
        const NODE: SerNode = SerNode::new_arry(Self::SER_FIELDS);
    }
    impl<T: Serialize, U: Serialize, V: Serialize, W: Serialize, X: Serialize> TupExtSer
        for (T, U, V, W, X)
    {
        const SER_FIELDS: &'static [SerField] = &[
            SerField {
                offset: core::mem::offset_of!((T, U, V, W, X), 0),
                node: &T::NODE,
            },
            SerField {
                offset: core::mem::offset_of!((T, U, V, W, X), 1),
                node: &U::NODE
            },
            SerField {
                offset: core::mem::offset_of!((T, U, V, W, X), 2),
                node: &V::NODE,
            },
            SerField {
                offset: core::mem::offset_of!((T, U, V, W, X), 3),
                node: &W::NODE,
            },
            SerField {
                offset: core::mem::offset_of!((T, U, V, W, X), 4),
                node: &X::NODE,
            },
        ];
    }

    unsafe impl<T: Serialize, U: Serialize, V: Serialize, W: Serialize, X: Serialize, Y: Serialize>
        Serialize for (T, U, V, W, X, Y)
    {
        const NODE: SerNode = SerNode::new_arry(Self::SER_FIELDS);
    }
    impl<T: Serialize, U: Serialize, V: Serialize, W: Serialize, X: Serialize, Y: Serialize>
        TupExtSer for (T, U, V, W, X, Y)
    {
        const SER_FIELDS: &'static [SerField] = &[
            SerField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y), 0),
                node: &T::NODE,
            },
            SerField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y), 1),
                node: &U::NODE
            },
            SerField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y), 2),
                node: &V::NODE,
            },
            SerField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y), 3),
                node: &W::NODE,
            },
            SerField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y), 4),
                node: &X::NODE,
            },
            SerField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y), 5),
                node: &Y::NODE,
            },
        ];
    }

    unsafe impl<
            T: Serialize,
            U: Serialize,
            V: Serialize,
            W: Serialize,
            X: Serialize,
            Y: Serialize,
            Z: Serialize,
        > Serialize for (T, U, V, W, X, Y, Z)
    {
        const NODE: SerNode = SerNode::new_arry(Self::SER_FIELDS);
    }
    impl<
            T: Serialize,
            U: Serialize,
            V: Serialize,
            W: Serialize,
            X: Serialize,
            Y: Serialize,
            Z: Serialize,
        > TupExtSer for (T, U, V, W, X, Y, Z)
    {
        const SER_FIELDS: &'static [SerField] = &[
            SerField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y, Z), 0),
                node: &T::NODE,
            },
            SerField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y, Z), 1),
                node: &U::NODE
            },
            SerField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y, Z), 2),
                node: &V::NODE,
            },
            SerField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y, Z), 3),
                node: &W::NODE,
            },
            SerField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y, Z), 4),
                node: &X::NODE,
            },
            SerField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y, Z), 5),
                node: &Y::NODE,
            },
            SerField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y, Z), 6),
                node: &Z::NODE,
            },
        ];
    }

    #[inline]
    pub unsafe fn deser_nothing(_stream: &mut DeserStream, _base: NonNull<()>) -> Result<(), ()> {
        Ok(())
    }

    #[inline]
    pub unsafe fn deser_bool(stream: &mut DeserStream, base: NonNull<()>) -> Result<(), ()> {
        match stream.pop_one() {
            Ok(0) => base.cast::<bool>().as_ptr().write(false),
            Ok(1) => base.cast::<bool>().as_ptr().write(true),
            _ => return Err(()),
        }
        Ok(())
    }

    #[inline]
    pub unsafe fn deser_u8(stream: &mut DeserStream, base: NonNull<()>) -> Result<(), ()> {
        if let Ok(val) = stream.pop_one() {
            base.cast::<u8>().as_ptr().write(val);
            Ok(())
        } else {
            Err(())
        }
    }

    #[inline]
    pub unsafe fn deser_u16(stream: &mut DeserStream, base: NonNull<()>) -> Result<(), ()> {
        if let Ok(val) = try_take_varint_u16(stream) {
            base.cast::<u16>().as_ptr().write(val);
            Ok(())
        } else {
            Err(())
        }
    }

    #[inline]
    pub unsafe fn deser_u32(stream: &mut DeserStream, base: NonNull<()>) -> Result<(), ()> {
        if let Ok(val) = try_take_varint_u32(stream) {
            base.cast::<u32>().as_ptr().write(val);
            Ok(())
        } else {
            Err(())
        }
    }

    #[inline]
    pub unsafe fn deser_u64(stream: &mut DeserStream, base: NonNull<()>) -> Result<(), ()> {
        if let Ok(val) = try_take_varint_u64(stream) {
            base.cast::<u64>().as_ptr().write(val);
            Ok(())
        } else {
            Err(())
        }
    }

    #[inline]
    pub unsafe fn deser_u128(stream: &mut DeserStream, base: NonNull<()>) -> Result<(), ()> {
        if let Ok(val) = try_take_varint_u128(stream) {
            base.cast::<u128>().as_ptr().write(val);
            Ok(())
        } else {
            Err(())
        }
    }

    #[inline]
    pub unsafe fn deser_usize(stream: &mut DeserStream, base: NonNull<()>) -> Result<(), ()> {
        if let Ok(val) = try_take_varint_usize(stream) {
            base.cast::<usize>().as_ptr().write(val);
            Ok(())
        } else {
            Err(())
        }
    }

    #[inline]
    pub unsafe fn deser_f32(stream: &mut DeserStream, base: NonNull<()>) -> Result<(), ()> {
        if let Ok(bytes) = stream.pop_n(size_of::<f32>()) {
            let mut buf = [0u8; size_of::<f32>()];
            buf.copy_from_slice(bytes);
            let val = f32::from_le_bytes(buf);
            base.cast::<f32>().as_ptr().write(val);
            Ok(())
        } else {
            Err(())
        }
    }

    #[inline]
    pub unsafe fn deser_f64(stream: &mut DeserStream, base: NonNull<()>) -> Result<(), ()> {
        if let Ok(bytes) = stream.pop_n(size_of::<f64>()) {
            let mut buf = [0u8; size_of::<f64>()];
            buf.copy_from_slice(bytes);
            let val = f64::from_le_bytes(buf);
            base.cast::<f64>().as_ptr().write(val);
            Ok(())
        } else {
            Err(())
        }
    }

    #[inline]
    pub unsafe fn deser_i8(stream: &mut DeserStream, base: NonNull<()>) -> Result<(), ()> {
        if let Ok(val) = stream.pop_one() {
            base.cast::<i8>().as_ptr().write(val as i8);
            Ok(())
        } else {
            Err(())
        }
    }

    #[inline]
    pub unsafe fn deser_i16(stream: &mut DeserStream, base: NonNull<()>) -> Result<(), ()> {
        if let Ok(val) = try_take_varint_u16(stream) {
            let val = de_zig_zag_i16(val);
            base.cast::<i16>().as_ptr().write(val);
            Ok(())
        } else {
            Err(())
        }
    }

    #[inline]
    pub unsafe fn deser_i32(stream: &mut DeserStream, base: NonNull<()>) -> Result<(), ()> {
        if let Ok(val) = try_take_varint_u32(stream) {
            let val = de_zig_zag_i32(val);
            base.cast::<i32>().as_ptr().write(val);
            Ok(())
        } else {
            Err(())
        }
    }

    #[inline]
    pub unsafe fn deser_i64(stream: &mut DeserStream, base: NonNull<()>) -> Result<(), ()> {
        if let Ok(val) = try_take_varint_u64(stream) {
            let val = de_zig_zag_i64(val);
            base.cast::<i64>().as_ptr().write(val);
            Ok(())
        } else {
            Err(())
        }
    }

    #[inline]
    pub unsafe fn deser_i128(stream: &mut DeserStream, base: NonNull<()>) -> Result<(), ()> {
        if let Ok(val) = try_take_varint_u128(stream) {
            let val = de_zig_zag_i128(val);
            base.cast::<i128>().as_ptr().write(val);
            Ok(())
        } else {
            Err(())
        }
    }

    #[inline]
    pub unsafe fn deser_isize(stream: &mut DeserStream, base: NonNull<()>) -> Result<(), ()> {
        if let Ok(val) = try_take_varint_usize(stream) {
            #[cfg(target_pointer_width = "16")]
            let val = de_zig_zag_i16(val as u16) as isize;
            #[cfg(target_pointer_width = "32")]
            let val = de_zig_zag_i32(val as u32) as isize;
            #[cfg(target_pointer_width = "64")]
            let val = de_zig_zag_i64(val as u64) as isize;
            base.cast::<isize>().as_ptr().write(val);
            Ok(())
        } else {
            Err(())
        }
    }

    #[cfg(feature = "std")]
    #[inline]
    pub unsafe fn deser_string(stream: &mut DeserStream, base: NonNull<()>) -> Result<(), ()> {
        let mut len = MaybeUninit::<usize>::uninit();
        deser_usize(stream, NonNull::from(&mut len).cast())?;
        let len = len.assume_init();
        let bytes = stream.pop_n(len)?;
        let utf = core::str::from_utf8(bytes).map_err(drop)?;
        let s = utf.to_string();
        base.cast::<String>().as_ptr().write(s);
        Ok(())
    }

    #[cfg(feature = "std")]
    #[inline]
    pub unsafe fn deser_vec<T: Deserialize>(
        stream: &mut DeserStream,
        base: NonNull<()>,
    ) -> Result<(), ()> {
        let mut len = MaybeUninit::<usize>::uninit();
        deser_usize(stream, NonNull::from(&mut len).cast())?;
        let len = len.assume_init();

        let mut out = Vec::<T>::with_capacity(len);
        let elems = out.spare_capacity_mut();
        for elem in elems.iter_mut().take(len) {
            deser_node(stream, NonNull::from(elem).cast(), &T::NODE)?;
        }

        out.set_len(len);
        base.cast::<Vec<T>>().as_ptr().write(out);
        Ok(())
    }

    #[inline]
    pub unsafe fn deser_arr<T: Deserialize, const N: usize>(
        stream: &mut DeserStream,
        base: NonNull<()>,
    ) -> Result<(), ()> {
        let mut cursor: *mut T = base.as_ptr().cast();
        let end = cursor.wrapping_add(N);
        while cursor != end {
            deser_node(stream, NonNull::new_unchecked(cursor).cast(), &T::NODE)?;
            cursor = cursor.wrapping_add(1);
        }
        Ok(())
    }

    #[inline]
    pub unsafe fn deser_option<T: Deserialize>(
        stream: &mut DeserStream,
        base: NonNull<()>,
    ) -> Result<(), ()> {
        let mut disc = MaybeUninit::<bool>::uninit();
        deser_bool(stream, NonNull::from(&mut disc).cast())?;
        let disc = disc.assume_init();

        if disc {
            let mut out = MaybeUninit::<T>::uninit();
            deser_fields_ref(stream, &mut out)?;
            base.cast::<Option<T>>()
                .as_ptr()
                .write(Some(out.assume_init()));
        } else {
            base.cast::<Option<T>>().as_ptr().write(None);
        }

        Ok(())
    }

    unsafe impl Deserialize for bool {
        const NODE: DeserNode = DeserNode::new_custom(deser_bool);
    }

    unsafe impl Deserialize for u8 {
        const NODE: DeserNode = DeserNode::new_custom(deser_u8);
    }

    unsafe impl Deserialize for u16 {
        const NODE: DeserNode = DeserNode::new_custom(deser_u16);
    }

    unsafe impl Deserialize for u32 {
        const NODE: DeserNode = DeserNode::new_custom(deser_u32);
    }

    unsafe impl Deserialize for u64 {
        const NODE: DeserNode = DeserNode::new_custom(deser_u64);
    }

    unsafe impl Deserialize for u128 {
        const NODE: DeserNode = DeserNode::new_custom(deser_u128);
    }

    unsafe impl Deserialize for usize {
        const NODE: DeserNode = DeserNode::new_custom(deser_usize);
    }

    unsafe impl Deserialize for f32 {
        const NODE: DeserNode = DeserNode::new_custom(deser_f32);
    }

    unsafe impl Deserialize for f64 {
        const NODE: DeserNode = DeserNode::new_custom(deser_f64);
    }

    unsafe impl Deserialize for i8 {
        const NODE: DeserNode = DeserNode::new_custom(deser_i8);
    }

    unsafe impl Deserialize for i16 {
        const NODE: DeserNode = DeserNode::new_custom(deser_i16);
    }

    unsafe impl Deserialize for i32 {
        const NODE: DeserNode = DeserNode::new_custom(deser_i32);
    }

    unsafe impl Deserialize for i64 {
        const NODE: DeserNode = DeserNode::new_custom(deser_i64);
    }

    unsafe impl Deserialize for i128 {
        const NODE: DeserNode = DeserNode::new_custom(deser_i128);
    }

    unsafe impl Deserialize for isize {
        const NODE: DeserNode = DeserNode::new_custom(deser_isize);
    }

    #[cfg(feature = "std")]
    unsafe impl Deserialize for String {
        const NODE: DeserNode = DeserNode::new_custom(deser_string);
    }

    #[cfg(feature = "std")]
    unsafe impl<T: Deserialize> Deserialize for Vec<T> {
        const NODE: DeserNode = DeserNode::new_custom(deser_vec::<T>);
    }

    unsafe impl<T: Deserialize, const N: usize> Deserialize for [T; N] {
        const NODE: DeserNode = DeserNode::new_custom(deser_arr::<T, N>);
    }

    unsafe impl<T: Deserialize> Deserialize for Option<T> {
        const NODE: DeserNode = DeserNode::new_custom(deser_option::<T>);
    }

    unsafe impl<T: Deserialize> Deserialize for (T,) {
        const NODE: DeserNode = DeserNode::new_arry(Self::DESER_FIELDS);
    }
    impl<T: Deserialize> TupExtDeser for (T,) {
        const DESER_FIELDS: &'static [DeserField] = &[DeserField {
            offset: core::mem::offset_of!((T,), 0),
            node: &T::NODE,
        }];
    }

    unsafe impl<T: Deserialize, U: Deserialize> Deserialize for (T, U) {
        const NODE: DeserNode = DeserNode::new_arry(Self::DESER_FIELDS);
    }
    impl<T: Deserialize, U: Deserialize> TupExtDeser for (T, U) {
        const DESER_FIELDS: &'static [DeserField] = &[
            DeserField {
                offset: core::mem::offset_of!((T, U), 0),
                node: &T::NODE,
            },
            DeserField {
                offset: core::mem::offset_of!((T, U), 1),
                node: &U::NODE,
            },
        ];
    }

    unsafe impl<T: Deserialize, U: Deserialize, V: Deserialize> Deserialize for (T, U, V) {
        const NODE: DeserNode = DeserNode::new_arry(Self::DESER_FIELDS);
    }
    impl<T: Deserialize, U: Deserialize, V: Deserialize> TupExtDeser for (T, U, V) {
        const DESER_FIELDS: &'static [DeserField] = &[
            DeserField {
                offset: core::mem::offset_of!((T, U, V), 0),
                node: &T::NODE,
            },
            DeserField {
                offset: core::mem::offset_of!((T, U, V), 1),
                node: &U::NODE,
            },
            DeserField {
                offset: core::mem::offset_of!((T, U, V), 2),
                node: &V::NODE,
            },
        ];
    }

    unsafe impl<T: Deserialize, U: Deserialize, V: Deserialize, W: Deserialize> Deserialize
        for (T, U, V, W)
    {
        const NODE: DeserNode = DeserNode::new_arry(Self::DESER_FIELDS);
    }
    impl<T: Deserialize, U: Deserialize, V: Deserialize, W: Deserialize> TupExtDeser
        for (T, U, V, W)
    {
        const DESER_FIELDS: &'static [DeserField] = &[
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W), 0),
                node: &T::NODE,
            },
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W), 1),
                node: &U::NODE,
            },
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W), 2),
                node: &V::NODE,
            },
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W), 3),
                node: &W::NODE,
            },
        ];
    }

    unsafe impl<T: Deserialize, U: Deserialize, V: Deserialize, W: Deserialize, X: Deserialize>
        Deserialize for (T, U, V, W, X)
    {
        const NODE: DeserNode = DeserNode::new_arry(Self::DESER_FIELDS);
    }
    impl<T: Deserialize, U: Deserialize, V: Deserialize, W: Deserialize, X: Deserialize>
        TupExtDeser for (T, U, V, W, X)
    {
        const DESER_FIELDS: &'static [DeserField] = &[
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W, X), 0),
                node: &T::NODE,
            },
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W, X), 1),
                node: &U::NODE,
            },
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W, X), 2),
                node: &V::NODE,
            },
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W, X), 3),
                node: &W::NODE,
            },
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W, X), 4),
                node: &X::NODE,
            },
        ];
    }

    unsafe impl<
            T: Deserialize,
            U: Deserialize,
            V: Deserialize,
            W: Deserialize,
            X: Deserialize,
            Y: Deserialize,
        > Deserialize for (T, U, V, W, X, Y)
    {
        const NODE: DeserNode = DeserNode::new_arry(Self::DESER_FIELDS);
    }
    impl<
            T: Deserialize,
            U: Deserialize,
            V: Deserialize,
            W: Deserialize,
            X: Deserialize,
            Y: Deserialize,
        > TupExtDeser for (T, U, V, W, X, Y)
    {
        const DESER_FIELDS: &'static [DeserField] = &[
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y), 0),
                node: &T::NODE,
            },
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y), 1),
                node: &U::NODE,
            },
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y), 2),
                node: &V::NODE,
            },
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y), 3),
                node: &W::NODE,
            },
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y), 4),
                node: &X::NODE,
            },
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y), 5),
                node: &Y::NODE,
            },
        ];
    }

    unsafe impl<
            T: Deserialize,
            U: Deserialize,
            V: Deserialize,
            W: Deserialize,
            X: Deserialize,
            Y: Deserialize,
            Z: Deserialize,
        > Deserialize for (T, U, V, W, X, Y, Z)
    {
        const NODE: DeserNode = DeserNode::new_arry(Self::DESER_FIELDS);
    }
    impl<
            T: Deserialize,
            U: Deserialize,
            V: Deserialize,
            W: Deserialize,
            X: Deserialize,
            Y: Deserialize,
            Z: Deserialize,
        > TupExtDeser for (T, U, V, W, X, Y, Z)
    {
        const DESER_FIELDS: &'static [DeserField] = &[
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y, Z), 0),
                node: &T::NODE,
            },
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y, Z), 1),
                node: &U::NODE,
            },
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y, Z), 2),
                node: &V::NODE,
            },
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y, Z), 3),
                node: &W::NODE,
            },
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y, Z), 4),
                node: &X::NODE,
            },
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y, Z), 5),
                node: &Y::NODE,
            },
            DeserField {
                offset: core::mem::offset_of!((T, U, V, W, X, Y, Z), 6),
                node: &Z::NODE,
            },
        ];
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
    use core::mem::offset_of;

    #[derive(Debug, PartialEq)]
    struct Alpha {
        a: u8,
        b: u16,
        c: u32,
        d: i8,
        e: i16,
        f: i32,
    }

//     #[derive(Debug, PartialEq)]
//     struct Beta {
//         a: u8,
//         b: u16,
//         c: u32,
//         d: i8,
//         e: i16,
//         f: i32,
//     }

//     #[derive(Debug, PartialEq)]
//     enum Dolsot {
//         Bib(Alpha),
//         Bim(Beta),
//         Bap(u32),
//         Bowl,
//     }

//     // THESE ARE THE PARTS THAT WILL HAVE TO BE MACRO GENERATED
//     //
//     //

//     #[inline]
//     pub unsafe fn ser_dolsot(stream: &mut SerStream, base: NonNull<()>) -> Result<(), ()> {
//         let eref = base.cast::<Dolsot>().as_ref();
//         let (var, fun, valref): (u32, SerFunc, NonNull<()>) = match eref {
//             Dolsot::Bib(x) => (0, ser_fields::<Alpha>, NonNull::from(x).cast::<()>()),
//             Dolsot::Bim(x) => (1, ser_fields::<Beta>, NonNull::from(x).cast::<()>()),
//             Dolsot::Bap(x) => (2, impls::ser_u32, NonNull::from(x).cast::<()>()),
//             Dolsot::Bowl => (3, impls::ser_nothing, NonNull::<()>::dangling()),
//         };

//         // serialize the discriminant as a u32
//         if impls::ser_u32(stream, NonNull::from(&var).cast()).is_err() {
//             return Err(());
//         }

//         // Serialize the payload
//         (fun)(stream, valref)
//     }

//     #[inline]
//     pub unsafe fn deser_dolsot(stream: &mut DeserStream, base: NonNull<()>) -> Result<(), ()> {
//         let mut disc = MaybeUninit::<u32>::uninit();
//         let dolsot_ref = base.cast::<Dolsot>();
//         if impls::deser_u32(stream, NonNull::from(&mut disc).cast()).is_err() {
//             return Err(());
//         }
//         let disc = disc.assume_init();
//         match disc {
//             0 => {
//                 // Bib
//                 let mut val = MaybeUninit::<Alpha>::uninit();
//                 deser_fields::<Alpha>(stream, NonNull::from(&mut val).cast())?;
//                 dolsot_ref.as_ptr().write(Dolsot::Bib(val.assume_init()));
//             }
//             1 => {
//                 // Bim
//                 let mut val = MaybeUninit::<Beta>::uninit();
//                 deser_fields::<Beta>(stream, NonNull::from(&mut val).cast())?;
//                 dolsot_ref.as_ptr().write(Dolsot::Bim(val.assume_init()));
//             }
//             2 => {
//                 // Bap
//                 let mut val = MaybeUninit::<u32>::uninit();
//                 impls::deser_u32(stream, NonNull::from(&mut val).cast())?;
//                 dolsot_ref.as_ptr().write(Dolsot::Bap(val.assume_init()));
//             }
//             3 => dolsot_ref.as_ptr().write(Dolsot::Bowl),
//             _ => return Err(()),
//         }
//         Ok(())
//     }

    impl Alpha {
        const SER_FIELDS: &'static [SerField] = &[
            // TODO: It would be possibly more efficient to directly call the various `ser_xx` functions here
            // rather than using the monomorphized handler when we KNOW we have a primitive type
            SerField {
                offset: offset_of!(Alpha, a),
                node: &<u8 as Serialize>::NODE,
            },
            SerField {
                offset: offset_of!(Alpha, b),
                node: &<u16 as Serialize>::NODE,
            },
            SerField {
                offset: offset_of!(Alpha, c),
                node: &<u32 as Serialize>::NODE,
            },
            SerField {
                offset: offset_of!(Alpha, d),
                node: &<i8 as Serialize>::NODE,
            },
            SerField {
                offset: offset_of!(Alpha, e),
                node: &<i16 as Serialize>::NODE,
            },
            SerField {
                offset: offset_of!(Alpha, f),
                node: &<i32 as Serialize>::NODE,
            },
        ];
    }

    unsafe impl Serialize for Alpha {
        const NODE: SerNode = SerNode::new_arry(Alpha::SER_FIELDS);
    }

//     unsafe impl Serialize for Beta {
//         const FIELDS: &'static [SerField] = &[
//             // This is a cross check that the native `ser_xx` functions are the same as calling
//             // ser_fields even for primitives
//             SerField {
//                 offset: offset_of!(Beta, a),
//                 func: impls::ser_u8,
//             },
//             SerField {
//                 offset: offset_of!(Beta, b),
//                 func: impls::ser_u16,
//             },
//             SerField {
//                 offset: offset_of!(Beta, c),
//                 func: impls::ser_u32,
//             },
//             SerField {
//                 offset: offset_of!(Beta, d),
//                 func: impls::ser_i8,
//             },
//             SerField {
//                 offset: offset_of!(Beta, e),
//                 func: impls::ser_i16,
//             },
//             SerField {
//                 offset: offset_of!(Beta, f),
//                 func: impls::ser_i32,
//             },
//         ];
//     }

    impl Alpha {
        const DESER_FIELDS: &'static [DeserField] = &[
            // TODO: It would be possibly more efficient to directly call the various `deser_xx` functions here
            // rather than using the monomorphized handler when we KNOW we have a primitive type
            DeserField {
                offset: offset_of!(Alpha, a),
                node: &<u8 as Deserialize>::NODE,
            },
            DeserField {
                offset: offset_of!(Alpha, b),
                node: &<u16 as Deserialize>::NODE,
            },
            DeserField {
                offset: offset_of!(Alpha, c),
                node: &<u32 as Deserialize>::NODE,
            },
            DeserField {
                offset: offset_of!(Alpha, d),
                node: &<i8 as Deserialize>::NODE,
            },
            DeserField {
                offset: offset_of!(Alpha, e),
                node: &<i16 as Deserialize>::NODE,
            },
            DeserField {
                offset: offset_of!(Alpha, f),
                node: &<i32 as Deserialize>::NODE,
            },
        ];
    }

    unsafe impl Deserialize for Alpha {
        const NODE: DeserNode = DeserNode::new_arry(Alpha::DESER_FIELDS);
    }

//     unsafe impl Deserialize for Beta {
//         const FIELDS: &'static [DeserField] = &[
//             // This is a cross check that the native `ser_xx` functions are the same as calling
//             // deser_fields even for primitives
//             DeserField {
//                 offset: offset_of!(Beta, a),
//                 func: impls::deser_u8,
//             },
//             DeserField {
//                 offset: offset_of!(Beta, b),
//                 func: impls::deser_u16,
//             },
//             DeserField {
//                 offset: offset_of!(Beta, c),
//                 func: impls::deser_u32,
//             },
//             DeserField {
//                 offset: offset_of!(Beta, d),
//                 func: impls::deser_i8,
//             },
//             DeserField {
//                 offset: offset_of!(Beta, e),
//                 func: impls::deser_i16,
//             },
//             DeserField {
//                 offset: offset_of!(Beta, f),
//                 func: impls::deser_i32,
//             },
//         ];
//     }

//     unsafe impl Serialize for Dolsot {
//         const FIELDS: &'static [SerField] = &[SerField {
//             offset: 0,
//             func: ser_dolsot,
//         }];
//     }

//     unsafe impl Deserialize for Dolsot {
//         const FIELDS: &'static [DeserField] = &[DeserField {
//             offset: 0,
//             func: deser_dolsot,
//         }];
//     }

//     //
//     //
//     // END OF MACRO GENERATION

//     #[test]
//     fn smoke_enum() {
//         let a = Dolsot::Bim(Beta {
//             a: 1,
//             b: 256,
//             c: 65536,
//             d: -1,
//             e: -129,
//             f: -32769,
//         });

//         let mut outa = [0u8; 64];
//         let mut sers = SerStream::from(outa.as_mut_slice());
//         unsafe {
//             ser_fields_ref(&mut sers, &a).unwrap();
//         }
//         let remain = sers.remain();
//         let used = outa.len() - remain;
//         assert_eq!(used, 13);
//         assert_eq!(
//             &outa[..used],
//             &[1, 1, 128, 2, 128, 128, 4, 255, 129, 2, 129, 128, 4]
//         );

//         // -

//         let mut desers = DeserStream::from(&outa[..used]);
//         let mut out = MaybeUninit::<Dolsot>::uninit();
//         unsafe {
//             deser_fields_ref(&mut desers, &mut out).unwrap();
//         }
//         let remain = desers.remain();
//         assert_eq!(remain, 0);
//         let out = unsafe { out.assume_init() };
//         assert_eq!(
//             Dolsot::Bim(Beta {
//                 a: 1,
//                 b: 256,
//                 c: 65536,
//                 d: -1,
//                 e: -129,
//                 f: -32769,
//             }),
//             out,
//         );
//     }

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
        unsafe {
            ser_fields_ref(&mut sers, &a).unwrap();
        }
        let remain = sers.remain();
        let used = outa.len() - remain;
        assert_eq!(used, 12);
        assert_eq!(
            &outa[..used],
            &[1, 128, 2, 128, 128, 4, 255, 129, 2, 129, 128, 4]
        );

        // let b = Beta {
        //     a: 1,
        //     b: 256,
        //     c: 65536,
        //     d: -1,
        //     e: -129,
        //     f: -32769,
        // };

        // let mut outb = [0u8; 64];
        // let mut sers = SerStream::from(outb.as_mut_slice());
        // unsafe {
        //     ser_fields_ref(&mut sers, &b).unwrap();
        // }
        // let remain = sers.remain();
        // let used = outb.len() - remain;
        // assert_eq!(used, 12);
        // assert_eq!(
        //     &outb[..used],
        //     &[1, 128, 2, 128, 128, 4, 255, 129, 2, 129, 128, 4]
        // );
    }

    #[test]
    fn smoke_deser() {
        let bytes = &[1, 128, 2, 128, 128, 4, 255, 129, 2, 129, 128, 4];

        let mut desers = DeserStream::from(bytes.as_slice());
        let mut out = MaybeUninit::<Alpha>::uninit();
        unsafe {
            deser_fields_ref(&mut desers, &mut out).unwrap();
        }
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

//         let mut desers = DeserStream::from(bytes.as_slice());
//         let mut out = MaybeUninit::<Beta>::uninit();
//         unsafe {
//             deser_fields_ref(&mut desers, &mut out).unwrap();
//         }
//         let remain = desers.remain();
//         assert_eq!(remain, 0);
//         let out = unsafe { out.assume_init() };
//         assert_eq!(
//             Beta {
//                 a: 1,
//                 b: 256,
//                 c: 65536,
//                 d: -1,
//                 e: -129,
//                 f: -32769,
//             },
//             out,
//         );
    }
}
