use rand::seq::SliceRandom;
use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};

const TYPES_TO_GEN: usize = 128;
const SEED: u64 = 0x1234_5678_9ABC_DEF0;
const BASE_TYS: &[&str] = &[
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32",
    "f64", "bool",
];

type GenTyFn = fn(usize, &mut ChaCha20Rng, &[String]) -> (String, String);

fn main() {
    let mut rng = ChaCha20Rng::seed_from_u64(SEED);
    let mut tys: Vec<String> = BASE_TYS.iter().map(ToString::to_string).collect();
    let mut generated: Vec<String> = vec![];

    let generators: &[GenTyFn] = &[
        gen_struct,
        gen_struct,
        gen_enum,
        gen_enum,
        gen_array,
        gen_option,
        gen_tuple,
        gen_tuple_struct,
    ];

    for i in 0..TYPES_TO_GEN {
        let gen_fn = generators.choose(&mut rng).unwrap();
        let (tyname, tybody) = (gen_fn)(i, &mut rng, &tys);
        tys.push(tyname);
        generated.push(tybody);
    }

    println!("//! Generated by `stress-gen` - do not edit!");
    println!();
    for g in generated {
        println!("/// generated");
        println!("{g}");
        println!();
    }

    println!(r#"
use core::fmt::Write;

struct SliceWrite<'a> {{
    remain: &'a mut [u8],
    used: usize
}}

impl<'a> Write for SliceWrite<'a> {{
    #[inline]
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {{
        let slen = s.len();
        if slen > self.remain.len() {{
            return Err(core::fmt::Error);
        }}
        let rem = core::mem::take(&mut self.remain);
        let (now, later) = rem.split_at_mut(slen);
        now.copy_from_slice(s.as_bytes());
        self.remain = later;
        self.used += slen;
        Ok(())
    }}
}}

#[inline]
pub(crate) fn debug<T: core::fmt::Debug>(
    t: &T,
    out_buf: &mut [u8],
) -> Result<usize, ()> {{
    let mut sw = SliceWrite {{ remain: out_buf, used: 0 }};
    write!(&mut sw, "{{t:?}}").map_err(drop)?;
    Ok(sw.used)
}}

#[cfg(feature = "serde")]
#[inline]
fn deserialize<T: ::serde::de::DeserializeOwned>(
    in_buf: &[u8],
) -> Result<T, ()> {{
    ::postcard::from_bytes::<T>(in_buf).map_err(drop)
}}

#[cfg(feature = "postcard-forth")]
#[inline]
fn deserialize<T: ::postcard_forth::Deserialize>(
    in_buf: &[u8],
) -> Result<T, ()> {{
    let mut desers = ::postcard_forth::DeserStream::from(in_buf);
    let mut out = core::mem::MaybeUninit::<T>::uninit();
    unsafe {{
        ::postcard_forth::deser_fields_ref(&mut desers, &mut out).map_err(drop)?;
    }}
    Ok(unsafe {{ out.assume_init() }})
}}
    "#);

    println!(r#"
pub(crate) fn round_trip_all<FI, FD, FO>(
    in_buf: &mut [u8],
    out_buf: &mut [u8],
    mut recv_fn: FI,
    debg_fn: FD,
    _send_fn: FO,
) -> Result<(), ()>
where
    FI: FnMut(&mut [u8]),
    FD: Fn(&[u8]),
    FO: Fn(&[u8]),
{{
    "#);

    for ty in tys.iter().skip(BASE_TYS.len()) {
        println!(r#"
    {{
        recv_fn(in_buf);
        let t: {ty} = deserialize(in_buf)?;
        let len = debug::<{ty}>(&t, out_buf)?;
        debg_fn(&out_buf[..len]);
    }}
        "#);
    }
    println!("    Ok(())");
    println!("}}");

}

fn gen_struct(idx: usize, rng: &mut ChaCha20Rng, tys: &[String]) -> (String, String) {
    let tyname = format!("GenStruct{idx:08X}");
    let mut out = String::new();
    out += "#[derive(Debug)]\n";
    out += r#"#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]"#;
    out += "\n";
    out += r#"#[cfg_attr(feature = "postcard-forth", derive(::postcard_forth_derive::Serialize, ::postcard_forth_derive::Deserialize))]"#;
    out += "\n";

    let num_fields = rng.next_u32() & 0b111;

    if num_fields == 0 {
        out += &format!("struct {tyname};");
        return (tyname, out);
    }

    out += &format!("struct {tyname} {{\n");

    for fidx in 0..num_fields {
        let fieldty = tys.choose(rng).unwrap().as_str();
        out += &format!("    field{fidx:02X}: {fieldty},\n");
    }

    out += "}";

    (tyname, out)
}

fn gen_array(idx: usize, rng: &mut ChaCha20Rng, tys: &[String]) -> (String, String) {
    let tyname = format!("GenArray{idx:08X}");
    let num_items = (rng.next_u32() & (32 - 1)) + 1;
    let arrty = tys.choose(rng).unwrap().as_str();
    let out = format!("type {tyname} = [{arrty}; {num_items}];");
    (tyname, out)
}

fn gen_option(idx: usize, rng: &mut ChaCha20Rng, tys: &[String]) -> (String, String) {
    let tyname = format!("GenOpt{idx:08X}");
    let optty = tys.choose(rng).unwrap().as_str();
    let out = format!("type {tyname} = Option<{optty}>;");
    (tyname, out)
}

fn gen_tuple(idx: usize, rng: &mut ChaCha20Rng, tys: &[String]) -> (String, String) {
    let tyname = format!("GenTup{idx:08X}");
    let num_items = (rng.next_u32() & (4 - 1)) + 1;
    let mut out = format!("type {tyname} = (");
    for _ in 0..num_items {
        let tupty = tys.choose(rng).unwrap().as_str();
        out += tupty;
        out += ", ";
    }
    out += ");";
    (tyname, out)
}

fn gen_tuple_struct(idx: usize, rng: &mut ChaCha20Rng, tys: &[String]) -> (String, String) {
    let tyname = format!("GenTupStruct{idx:08X}");
    let num_items = (rng.next_u32() & (4 - 1)) + 1;

    let mut out = String::new();
    out += "#[derive(Debug)]\n";
    out += r#"#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]"#;
    out += "\n";
    out += r#"#[cfg_attr(feature = "postcard-forth", derive(::postcard_forth_derive::Serialize, ::postcard_forth_derive::Deserialize))]"#;
    out += "\n";

    out += &format!("struct {tyname}(");

    for _ in 0..num_items {
        let tupty = tys.choose(rng).unwrap().as_str();
        out += tupty;
        out += ", ";
    }
    out += ");";
    (tyname, out)
}

fn gen_enum(idx: usize, rng: &mut ChaCha20Rng, tys: &[String]) -> (String, String) {
    let tyname = format!("GenEnum{idx:08X}");
    let mut out = String::new();
    out += "#[derive(Debug)]\n";
    out += r#"#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]"#;
    out += "\n";
    out += r#"#[cfg_attr(feature = "postcard-forth", derive(::postcard_forth_derive::Serialize, ::postcard_forth_derive::Deserialize))]"#;
    out += "\n";

    out += &format!("enum {tyname} {{\n");

    let num_items = (rng.next_u32() & (16 - 1)) + 1;
    let gen_vars: &[GenVarFn] = &[
        gen_var_empty,
        gen_var_newtype,
        gen_var_tuple,
        gen_var_struct,
    ];

    for vidx in 0..num_items {
        let var_func = gen_vars.choose(rng).unwrap();
        let var_body = (var_func)(vidx as usize, rng, tys);
        out += &format!("    {var_body}\n");
    }

    out += "}";

    (tyname, out)
}

type GenVarFn = fn(usize, &mut ChaCha20Rng, &[String]) -> String;

fn gen_var_empty(idx: usize, _rng: &mut ChaCha20Rng, _tys: &[String]) -> String {
    format!("VarEmpty{idx:02X},")
}

fn gen_var_newtype(idx: usize, rng: &mut ChaCha20Rng, tys: &[String]) -> String {
    let tupty = tys.choose(rng).unwrap().as_str();
    format!("VarNewTy{idx:02X}({tupty}),")
}

fn gen_var_tuple(idx: usize, rng: &mut ChaCha20Rng, tys: &[String]) -> String {
    let num_items = (rng.next_u32() & (4 - 1)) + 1;
    let mut out = format!("VarTuple{idx:02X}(");
    for _ in 0..num_items {
        let tupty = tys.choose(rng).unwrap().as_str();
        out += tupty;
        out += ", ";
    }
    out += "),";
    out
}

fn gen_var_struct(idx: usize, rng: &mut ChaCha20Rng, tys: &[String]) -> String {
    let num_items = (rng.next_u32() & (4 - 1)) + 1;
    let mut out = format!("VarStruct{idx:02X} {{\n");
    for fidx in 0..num_items {
        let fieldty = tys.choose(rng).unwrap().as_str();
        out += &format!("        field{fidx:02X}: {fieldty},\n");
    }
    out += "    },";
    out
}