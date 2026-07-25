// deno-lint-ignore-file require-jsdoc
import { escapeHtml, isModifiedClick } from '../utilities.js';

export function test_escapeHtml_handles_special_characters() {
    assertSame(escapeHtml('<div>'), '&lt;div&gt;');
    assertSame(escapeHtml('hello & world'), 'hello &amp; world');
    assertSame(escapeHtml('"test"'), '&quot;test&quot;');
    assertSame(escapeHtml('\'test\''), '&#039;test&#039;');
}

export function test_escapeHtml_handles_empty_or_null() {
    assertSame(escapeHtml(null), '');
    assertSame(escapeHtml(undefined), '');
    assertSame(escapeHtml(''), '');
}

export function test_isModifiedClick_returns_true_with_modifiers() {
    assert(isModifiedClick({ ctrlKey: true }));
    assert(isModifiedClick({ metaKey: true }));
    assert(isModifiedClick({ shiftKey: true }));
    assert(isModifiedClick({ altKey: true }));
}

export function test_isModifiedClick_returns_false_without_modifiers() {
    assert(!isModifiedClick({ ctrlKey: false, metaKey: false, shiftKey: false, altKey: false }));
}
