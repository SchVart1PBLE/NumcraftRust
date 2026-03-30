#[inline(always)]
pub fn div_floor(a: isize, b: isize) -> isize {
    let d = a / b;
    let r = a % b;
    // If remainder is nonzero and sign differs, subtract 1
    if (r != 0) && ((r < 0) != (b < 0)) { d - 1 } else { d }
}

#[inline(always)]
pub fn mod_floor(a: isize, b: isize) -> isize {
    let r = a % b;
    if (r != 0) && ((r < 0) != (b < 0)) { r + b } else { r }
}
