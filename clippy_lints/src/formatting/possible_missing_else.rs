use super::POSSIBLE_MISSING_ELSE;
use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::source::SpanExt;
use rustc_ast::{Expr, ExprKind};
use rustc_errors::Applicability;
use rustc_lint::EarlyContext;
use rustc_span::{Span, SyntaxContext};

pub(super) fn check(cx: &EarlyContext<'_>, ctxt: SyntaxContext, first: &Expr, second: &Expr) {
    if matches!(first.kind, ExprKind::If(..))
    && matches!(second.kind, ExprKind::If(..) | ExprKind::Block(..))
    && let first_data = first.span.data()
    && let second_data = second.span.data()
    && first_data.ctxt == ctxt
    && second_data.ctxt == ctxt
    && let Some(mut check_range) = first_data.get_source_range(cx)
    && check_range.current_text().is_some_and(|src| src.starts_with("if") && src.ends_with('}'))
    && let Some(check_range) = check_range.set_range_between_other(second_data)
    // Only lint when the end of the first expression and the start of the
    // second are on the same line without anything in between.
    && check_range.current_text().is_some_and(|src| src.chars().all(|c| c != '\n' && c.is_whitespace()))
    {
        let range = check_range.source_range();
        let sp = Span::new(range.start, range.end, first_data.ctxt, first_data.parent);
        span_lint_and_then(
            cx,
            POSSIBLE_MISSING_ELSE,
            sp,
            "this is formatted as though there should be an `else`",
            |diag| {
                diag.span_suggestion(sp, "add an `else`", " else ", Applicability::MaybeIncorrect)
                    .span_suggestion(
                        sp,
                        "add a line break",
                        format!("\n{}", check_range.get_line_indent()),
                        Applicability::MaybeIncorrect,
                    );
            },
        );
    }
}
