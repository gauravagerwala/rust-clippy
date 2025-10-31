use super::SUSPICIOUS_ASSIGNMENT_FORMATTING;
use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::source::SpanExt;
use rustc_ast::{Expr, ExprKind};
use rustc_errors::Applicability;
use rustc_lint::{EarlyContext, LintContext};
use rustc_span::Span;

pub(super) fn check(cx: &EarlyContext<'_>, assign: &Expr, rhs: &Expr, op_sp: Span) {
    if let ExprKind::Unary(op, _) = rhs.kind
        && let assign_data = assign.span.data()
        && rhs.span.ctxt() == assign_data.ctxt
        && let op_data = op_sp.data()
        && op_data.ctxt == assign_data.ctxt
        && let op_str = op.as_str()
        && let Some(mut check_range) = op_data.get_source_range(cx)
        && let Some(check_range) = check_range.set_end_if_after(assign_data.hi)
        && let Some(check_range) = check_range.edit_range(|src, range| {
            if let Some(src) = src.get(range.clone())
                && let Some(src) = src.strip_prefix('=')
                && let Some(src) = src.strip_prefix(op_str)
                && src.starts_with(|c: char| c.is_whitespace())
            {
                Some(range.start..range.start + 2)
            } else {
                None
            }
        })
        && let lint_range = check_range.source_range()
        && let Some(sep_range) = check_range.add_trailing_whitespace()
        && !assign_data.ctxt.in_external_macro(cx.sess().source_map())
    {
        let sep_range = sep_range.source_range();
        let lint_sp = Span::new(lint_range.start, lint_range.end, assign_data.ctxt, assign_data.parent);
        span_lint_and_then(
            cx,
            SUSPICIOUS_ASSIGNMENT_FORMATTING,
            lint_sp,
            "this looks similar to a compound assignment operator",
            |diag| {
                diag.span_suggestion(
                    lint_sp,
                    "reverse the characters",
                    format!("{op_str}="),
                    Applicability::MaybeIncorrect,
                )
                .span_suggestion(
                    Span::new(sep_range.start, sep_range.end, assign_data.ctxt, assign_data.parent),
                    "separate the characters",
                    format!("= {op_str}"),
                    Applicability::MaybeIncorrect,
                );
            },
        );
    }
}
