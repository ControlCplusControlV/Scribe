use include_dir::{include_dir, Dir};
use itertools::Itertools;
use primitive_types::U256;

//Function to output Miden assembly to convert a u256 struct, into eight 32bit segments and push them onto the stack
pub fn convert_u256_to_pushes(x: &U256) -> String {
    let mut bytes = [0u8; 32];
    x.to_little_endian(&mut bytes);
    bytes
        .iter()
        .chunks(4)
        .into_iter()
        .collect::<Vec<_>>()
        .into_iter()
        .map(|bytes| {
            let mut stack_value: u32 = 0;
            for (i, bytes) in bytes.enumerate() {
                stack_value = stack_value | ((*bytes as u32) << ((i) * 8)) as u32
            }
            format!("push.{:<10}", stack_value)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn split_u256_to_u32s(x: &U256) -> Vec<u32> {
    let mut bytes = [0u8; 32];
    x.to_little_endian(&mut bytes);
    bytes
        .iter()
        .chunks(4)
        .into_iter()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|bytes| {
            let mut stack_value: u32 = 0;
            for (i, bytes) in bytes.enumerate() {
                stack_value = stack_value | ((*bytes as u32) << ((i) * 8)) as u32
            }
            stack_value
        })
        .collect::<Vec<_>>()
}

pub fn join_u32s_to_u256(x: Vec<u32>) -> U256 {
    let u256_bytes = x
        .iter()
        .take(8)
        .flat_map(|x| x.to_be_bytes())
        .collect::<Vec<_>>();

    U256::from_big_endian(&u256_bytes)
}

pub fn load_all_procs() -> String {
    static MASM_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/miden_asm");
    MASM_DIR
        .files()
        .filter_map(|file| {
            if file.path().extension().unwrap().to_str() == Some("masm") {
                return file.contents_utf8();
            }
            None
        })
        .join("\n")
}
