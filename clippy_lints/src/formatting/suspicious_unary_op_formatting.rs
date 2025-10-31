use super::SUSPICIOUS_UNARY_OP_FORMATTING;
use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::source::SpanExt;
use rustc_ast::{BinOp, Expr, ExprKind};
use rustc_errors::Applicability;
use rustc_lint::{EarlyContext, LintContext};
use rustc_span::Span;

pub(super) fn check(cx: &EarlyContext<'_>, bin_expr: &Expr, bin_op: &BinOp, rhs: &Expr) {
    if let ExprKind::Unary(un_op, _) = rhs.kind
        && let bin_op_data = bin_op.span.data()
        && bin_op_data.ctxt == bin_expr.span.ctxt()
        && let rhs_data = rhs.span.data()
        && rhs_data.ctxt == bin_op_data.ctxt
        && let bin_op_str = bin_op.node.as_str()
        && let un_op_str = un_op.as_str()
        && let Some(mut check_range) = bin_op_data.get_source_range(cx)
        && let Some(check_range) = check_range.set_end_if_after(rhs_data.hi)
        && let Some(check_range) = check_range.edit_range(|src, range| {
            if let Some(src) = src.get(range.clone())
                && let Some(src) = src.strip_prefix(bin_op_str)
                && let Some(src) = src.strip_prefix(un_op_str)
                && src.starts_with(|c: char| c.is_whitespace())
            {
                Some(range.start..range.start + bin_op_str.len() + un_op_str.len())
            } else {
                None
            }
        })
        && let lint_range = check_range.source_range()
        && let Some(sugg_range) = check_range.add_trailing_whitespace()
        && !bin_op_data.ctxt.in_external_macro(cx.sess().source_map())
    {
        span_lint_and_then(
            cx,
            SUSPICIOUS_UNARY_OP_FORMATTING,
            Span::new(lint_range.start, lint_range.end, bin_op_data.ctxt, bin_op_data.parent),
            "this formatting makes the binary and unary operators look like a single operator",
            |diag| {
                let sugg_range = sugg_range.source_range();
                diag.span_suggestion(
                    Span::new(sugg_range.start, sugg_range.end, bin_op_data.ctxt, bin_op_data.parent),
                    "add a space between",
                    format!("{bin_op_str} {un_op_str}"),
                    if bin_op_data.ctxt.is_root() {
                        Applicability::MachineApplicable
                    } else {
                        Applicability::MaybeIncorrect
                    },
                );
            },
        );
    }
}
