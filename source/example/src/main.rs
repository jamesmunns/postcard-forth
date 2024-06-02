use std::mem::MaybeUninit;

use postcard_forth::{deser_fields_ref, ser_fields_ref, DeserStream, SerStream};
use postcard_forth_derive::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Alpha {
    a: u8,
    b: u16,
    c: u32,
    d: i8,
    e: i16,
    f: i32,
    g: Vec<u16>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Beta {
    a: u8,
    b: u16,
    c: u32,
    d: i8,
    e: i16,
    f: i32,
    g: Vec<u16>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Dolsot {
    Bib(Alpha),
    Bim(Beta),
    Bap(u32),
    Bowl,
    Sticks {
        left: u32,
        right: u8,
    }
}

fn main() {
    println!("Hello, world!");
    let a = Alpha {
        a: 1,
        b: 256,
        c: 65536,
        d: -1,
        e: -129,
        f: -32769,
        g: vec![1, 2, 3, 4],
    };

    let mut outa = [0u8; 64];
    let mut sers = SerStream::from(outa.as_mut_slice());
    unsafe {
        ser_fields_ref(&mut sers, &a).unwrap();
    }
    let remain = sers.remain();
    let used = outa.len() - remain;
    assert_eq!(used, 17);
    assert_eq!(
        &outa[..used],
        &[1, 128, 2, 128, 128, 4, 255, 129, 2, 129, 128, 4, 4, 1, 2, 3, 4]
    );

    // ---

    let bytes = &[1, 128, 2, 128, 128, 4, 255, 129, 2, 129, 128, 4, 4, 1, 2, 3, 4];

    let mut desers = DeserStream::from(bytes.as_slice());
    let mut out = MaybeUninit::<Alpha>::uninit();
    unsafe {
        deser_fields_ref(&mut desers, &mut out).unwrap();
    }
    let remain = desers.remain();
    assert_eq!(remain, 0);
    let out = unsafe { out.assume_init() };
    assert_eq!(
        a,
        out,
    );

    // ===

    let a = Dolsot::Bim(Beta {
        a: 1,
        b: 256,
        c: 65536,
        d: -1,
        e: -129,
        f: -32769,
        g: vec![1, 2, 3, 4],
    });

    let mut outa = [0u8; 64];
    let mut sers = SerStream::from(outa.as_mut_slice());
    unsafe {
        ser_fields_ref(&mut sers, &a).unwrap();
    }
    let remain = sers.remain();
    let used = outa.len() - remain;
    assert_eq!(used, 18);
    assert_eq!(
        &outa[..used],
        &[1, 1, 128, 2, 128, 128, 4, 255, 129, 2, 129, 128, 4, 4, 1, 2, 3, 4]
    );

    // ---

    let bytes = &[1, 1, 128, 2, 128, 128, 4, 255, 129, 2, 129, 128, 4, 4, 1, 2, 3, 4];

    let mut desers = DeserStream::from(bytes.as_slice());
    let mut out = MaybeUninit::<Dolsot>::uninit();
    unsafe {
        deser_fields_ref(&mut desers, &mut out).unwrap();
    }
    let remain = desers.remain();
    assert_eq!(remain, 0);
    let out = unsafe { out.assume_init() };
    assert_eq!(
        a,
        out,
    );

    println!("Passed!");
}
