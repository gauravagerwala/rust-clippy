use super::SUSPICIOUS_ELSE_FORMATTING;
use clippy_utils::diagnostics::span_lint_and_note;
use clippy_utils::source::SpanExt;
use clippy_utils::tokenize_with_text;
use core::mem;
use rustc_ast::{Block, Expr, ExprKind};
use rustc_lexer::TokenKind;
use rustc_lint::{EarlyContext, LintContext};
use rustc_span::Span;

pub(super) fn check(cx: &EarlyContext<'_>, expr: &Expr, then: &Block, else_: &Expr) {
    let then_data = then.span.data();
    if then_data.ctxt == expr.span.ctxt()
        && let else_data = else_.span.data()
        && then_data.ctxt == else_data.ctxt
        && let Some(mut check_range) = then_data.get_source_range(cx)
        && let Some(check_range) = check_range.set_range_between_other(else_data)
        && let is_else_block = matches!(else_.kind, ExprKind::Block(..))
        && check_range
            .current_text()
            .is_some_and(|src| check_else_formatting(src, is_else_block))
        && !then_data.ctxt.in_external_macro(cx.sess().source_map())
    {
        let else_desc = if is_else_block { "{..}" } else { "if" };
        let range = check_range.source_range();
        span_lint_and_note(
            cx,
            SUSPICIOUS_ELSE_FORMATTING,
            Span::new(range.start, range.end, then_data.ctxt, then_data.parent),
            format!("this is an `else {else_desc}` but the formatting might hide it"),
            None,
            format!(
                "to remove this lint, remove the `else` or remove the new line between \
                 `else` and `{else_desc}`",
            ),
        );
    }
}

fn check_else_formatting(src: &str, is_else_block: bool) -> bool {
    // Check for any of the following:
    // * A blank line between the end of the previous block and the `else`.
    // * A blank line between the `else` and the start of it's block.
    // * A block comment preceding the `else`, `if` or block if it's the first thing on the line.
    // * The `else` and `if` are on separate lines unless separated by multiple lines with every
    //   intervening line containing only block comments. This is due to rustfmt splitting
    //   `else/*comment*/if` into three lines.
    // * The `else` and it's block are on separate lines unless every intervening line containing only
    //   block comments. There must be one such line unless the `else` and the preceding block are on
    //   separate lines.
    let mut tokens = tokenize_with_text(src);
    let mut lf_count = 0;
    let mut skip_lf = false;
    loop {
        match tokens.next() {
            Some((TokenKind::Whitespace, text, _)) => match text.bytes().filter(|&c| c == b'\n').count() {
                0 => {},
                x => lf_count += x - usize::from(mem::replace(&mut skip_lf, false)),
            },
            Some((TokenKind::LineComment { .. }, _, _)) => skip_lf = lf_count != 0,
            Some((TokenKind::BlockComment { .. }, text, _)) => {
                if lf_count == 0 {
                    lf_count = usize::from(text.contains('\n'));
                }
                skip_lf = lf_count != 0;
            },
            Some((TokenKind::Ident, "else", _)) if skip_lf || lf_count > 1 => return true,
            Some((TokenKind::Ident, "else", _)) => break,
            _ => return false,
        }
    }
    let mut allow_lf = is_else_block && lf_count != 0;
    skip_lf = false;
    lf_count = 0;
    for (kind, text, _) in tokens {
        match kind {
            TokenKind::Whitespace => match text.bytes().filter(|&c| c == b'\n').count() {
                0 => {},
                x => lf_count += x - usize::from(mem::replace(&mut skip_lf, false)),
            },
            TokenKind::BlockComment { .. } => {
                skip_lf = lf_count != 0;
                allow_lf |= skip_lf;
            },
            TokenKind::LineComment { .. } => return true,
            _ => return false,
        }
    }
    skip_lf || lf_count > usize::from(allow_lf)
}
