// deno-lint-ignore-file require-jsdoc
export function test_addition() {
    assertSame(2 + 2, 4);
}

export function test_division_by_zero() {
    assertSame(1 / 0, Infinity);
}

export function test_math_errors() {
    assertThrows(() => {
        throw new TypeError("invalid");
    }, TypeError);
}
