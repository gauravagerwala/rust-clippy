use super::POSSIBLE_MISSING_COMMA;
use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::source::{SpanExt, walk_span_to_context};
use rustc_ast::{BinOpKind, Expr, ExprKind};
use rustc_errors::Applicability;
use rustc_lint::{EarlyContext, LintContext};
use rustc_span::{Pos, Span, SyntaxContext};

pub(super) fn check(cx: &EarlyContext<'_>, ctxt: SyntaxContext, e: &Expr) {
    if let ExprKind::Binary(op, lhs, rhs) = &e.kind
        && let e_data = e.span.data()
        && e_data.ctxt == ctxt
    {
        if matches!(
            op.node,
            BinOpKind::And | BinOpKind::Mul | BinOpKind::Sub | BinOpKind::BitAnd
        ) && let op_data = op.span.data()
            && op_data.ctxt == e_data.ctxt
            && let Some(mut check_range) = op_data.get_source_range(cx)
            && let Some(check_range) = check_range.set_end_if_after(e_data.hi)
            && let Some(src) = check_range.file_text().get(..check_range.range().end.to_usize())
            && let Some((pre_src, src)) = src.split_at_checked(check_range.range().start.to_usize())
            && let Some(src) = src.strip_prefix(op.node.as_str())
            && src.starts_with(|c: char| !c.is_whitespace() && c != '/')
            && pre_src.ends_with(|c: char| c.is_whitespace())
            && let Some(lhs_sp) = walk_span_to_context(lhs.span, ctxt)
            && !ctxt.in_external_macro(cx.sess().source_map())
        {
            span_lint_and_then(
                cx,
                POSSIBLE_MISSING_COMMA,
                op.span,
                "the is formatted like a unary operator, but it's parsed as a binary operator",
                |diag| {
                    diag.span_suggestion(
                        lhs_sp.shrink_to_hi(),
                        "add a comma before",
                        ",",
                        Applicability::MaybeIncorrect,
                    )
                    .span_suggestion(
                        Span::new(op_data.hi, op_data.hi, op_data.ctxt, op_data.parent),
                        "add a space after",
                        " ",
                        Applicability::MaybeIncorrect,
                    );
                },
            );
        }
        check(cx, ctxt, lhs);
        check(cx, ctxt, rhs);
    }
}
