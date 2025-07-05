use clippy_utils::diagnostics::span_lint_and_sugg;
use clippy_utils::is_never_expr;
use clippy_utils::source::{HasSession, SpanRangeExt, walk_span_to_context};
use clippy_utils::sym::clippy_utils;
use rustc_ast::LitKind;
use rustc_errors::Applicability;
use rustc_hir::*;
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::declare_lint_pass;

declare_clippy_lint! {
    /// ### What it does
    ///
    /// Warn about cases where `x && y` could be used in place of an if condition.
    ///
    /// ### Why is this bad?
    ///
    /// `x && y` is more standard as a construction, and makes it clearer that this is just an and.
    /// It is also less verbose.
    ///
    /// ### Example
    /// ```no_run
    /// # fn is_string_instrument(instrument: Instrument, is_in_orchestra: boolean) {
    /// #   if is_in_orchestra { is_orchestral_string_instrument(instrument) } else { false }
    /// # }
    ///
    /// ```
    ///
    /// Could be written
    ///
    /// ```no_run
    /// # fn is_string_instrument(instrument: Instrument, is_in_orchestra: boolean) {
    /// #   is_in_orchestra && is_orchestral_string_instrument(instrument)
    /// # }
    /// ```
    #[clippy::version = "1.89.0"]
    pub IFS_AS_LOGICAL_OPS,
    nursery,
    "default lint description"
}
declare_lint_pass!(IfsAsLogicalOps => [IFS_AS_LOGICAL_OPS]);

impl<'tcx> LateLintPass<'tcx> for IfsAsLogicalOps {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, e: &'tcx Expr<'tcx>) {
        if let ExprKind::If(cond, cond_inner, Some(els)) = e.kind
            && let ExprKind::Block(if_block, _label) = cond_inner.kind
            // Check if the if-block has only a return statement
            && if_block.stmts.len() == 0
            && let Some(if_expr) = if_block.expr
            // And that the else block consists of only the boolean 'false'.
            && let ExprKind::Block(else_block, _label) = els.kind
            && else_block.stmts.len() == 0
            && let Some(else_expr) = else_block.expr
            && let ExprKind::Lit(lit) = else_expr.kind
            && matches!(lit.node, LitKind::Bool(false))
            // We do not emit this lint if the expression diverges.
            && !cx.typeck_results().expr_ty(if_expr).is_never()
            // Make sure that the expression is only in a single macro context
            && let ctxt = e.span.ctxt()
            && ctxt == if_block.span.ctxt()
            && ctxt == else_block.span.ctxt()
            && ctxt == else_expr.span.ctxt()
            && ctxt == lit.span.ctxt()
            && !ctxt.in_external_macro(cx.tcx.sess().source_map())
        {
            if let Some(lhs_snippet) = walk_span_to_context(cond.span, ctxt)
                .map(|span| span.get_source_text(cx))
                .flatten()
                && let Some(rhs_snippet) = walk_span_to_context(if_expr.span, ctxt)
                    .map(|span| span.get_source_text(cx))
                    .flatten()
            {
                span_lint_and_sugg(
                    cx,
                    IFS_AS_LOGICAL_OPS,
                    e.span,
                    "if expression that could be written as a logical and expression",
                    "try",
                    format!("{lhs_snippet} && {rhs_snippet}"),
                    if ctxt.is_root() {
                        Applicability::MachineApplicable
                    } else {
                        Applicability::MaybeIncorrect
                    },
                );
            }
        }
    }
}
