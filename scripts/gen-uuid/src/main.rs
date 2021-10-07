fn main() {
    let mut buffer = uuid::Uuid::encode_buffer();
    let string = uuid::Uuid::new_v4().to_simple().encode_upper(&mut buffer);
    println!(
        "[0x{}, 0x{}, 0x{}, 0x{}]",
        &string[0..8],
        &string[8..16],
        &string[16..24],
        &string[24..32]
    );
}
