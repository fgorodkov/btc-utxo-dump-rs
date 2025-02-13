pub fn varint128_read(data: &[u8], offset: usize) -> (usize, Vec<u8>) {
    let mut result = Vec::new();
    // Loop through bytes starting from offset
    for &byte in data[offset..].iter() {
        result.push(byte);
        // Check if 8th bit is set (& 0b10000000)
        if byte & 0x80 == 0 {
            // Found byte without 8th bit set, return count and bytes
            return (result.len(), result);
        }
    }
    // Return empty result if we couldn't read properly
    (0, result)
}

pub fn varint128_decode(bytes: &[u8]) -> i64 {
    let mut n: i64 = 0;
    for &byte in bytes {
        // 1. Shift left 7 bits
        n <<= 7;
        // 2. Set last 7 bits from current byte
        n |= i64::from(byte & 0x7F);
        // 3. Add 1 if continuation bit (8th bit) is set
        if byte & 0x80 != 0 {
            n += 1;
        }
    }
    n
}

pub fn decompress_value(x: i64) -> i64 {
    if x == 0 {
        return 0;
    }

    let mut n = x - 1; // subtract 1 first
    let e = n % 10; // remainder mod 10
    n /= 10; // quotient mod 10

    if e < 9 {
        let d = n % 9; // remainder mod 9
        n /= 9; // quotient mod 9
        n = n * 10 + d + 1;
    } else {
        n = n + 1;
    }

    // Calculate 10^e
    let mut result = n;
    for _ in 0..e {
        result *= 10;
    }

    result
}
