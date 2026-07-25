// ---------------
// Booleans (logic gates)
// ---------------

pub fn or(a: bool, b: bool) -> bool {
    a || b
}
pub fn nor(a: bool, b: bool) -> bool {
    !(a || b)
}
pub fn nand(a: bool, b: bool) -> bool {
    !(a && b)
}
pub fn xor(a: bool, b: bool) -> bool {
    (a || b) && !(a && b)
}
pub fn xnor(a: bool, b: bool) -> bool {
    !xor(a, b)
}
pub fn is_true(v: bool) -> bool {
    v
}
pub fn is_false(v: bool) -> bool {
    !v
}
