use itertools::Itertools;
use primitive_types::U256;

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
