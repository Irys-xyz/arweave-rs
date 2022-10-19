pub fn copy_into_slice_32(m: &[u8]) -> [u8; 32] {
    let mut result: [u8; 32] = [0; 32];
    result.copy_from_slice(m);
    result
}

pub fn copy_into_slice_48(m: &[u8]) -> [u8; 48] {
    let mut result: [u8; 48] = [0; 48];
    result.copy_from_slice(m);
    result
}

pub fn concat_u8_48(left: [u8; 48], right: [u8; 48]) -> [u8; 96] {
    let mut iter = left.into_iter().chain(right);
    let result = [(); 96].map(|_| iter.next().expect("Could not get concat two arrays"));
    result
}
