use super::CHUNKS_EXACT_WITH_CONST_SIZE;
use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::higher::ForLoop;
use clippy_utils::msrvs::{self, Msrv};
use clippy_utils::source::snippet_with_applicability;
use clippy_utils::visitors::is_const_evaluatable;
use clippy_utils::{get_parent_expr, sym};
use rustc_errors::Applicability;
use rustc_hir::{Expr, Node, PatKind};
use rustc_lint::LateContext;
use rustc_middle::ty;
use rustc_span::{Span, Symbol};

pub(super) fn check<'tcx>(
    cx: &LateContext<'tcx>,
    recv: &'tcx Expr<'tcx>,
    arg: &'tcx Expr<'tcx>,
    expr: &'tcx Expr<'tcx>,
    call_span: Span,
    method_name: Symbol,
    msrv: Msrv,
) {
    let recv_ty = cx.typeck_results().expr_ty_adjusted(recv);
    if !matches!(recv_ty.kind(), ty::Ref(_, inner, _) if inner.is_slice()) {
        return;
    }

    if is_const_evaluatable(cx, arg) {
        if !msrv.meets(cx, msrvs::AS_CHUNKS) {
            return;
        }

        let suggestion_method = if method_name == sym::chunks_exact_mut {
            "as_chunks_mut"
        } else {
            "as_chunks"
        };

        let mut applicability = Applicability::MachineApplicable;
        let arg_str = snippet_with_applicability(cx, arg.span, "_", &mut applicability);

        let as_chunks = format_args!("{suggestion_method}::<{arg_str}>()");

        span_lint_and_then(
            cx,
            CHUNKS_EXACT_WITH_CONST_SIZE,
            call_span,
            format!("using `{method_name}` with a constant chunk size"),
            |diag| {
                if let Node::LetStmt(let_stmt) = cx.tcx.parent_hir_node(expr.hir_id) {
                    // The `ChunksExact(Mut)` struct is stored for later -- this likely means that the user intends to
                    // not only use it as an iterator, but also access the remainder using
                    // `(into_)remainder`. For now, just give a help message in this case.
                    // TODO: give a suggestion that replaces this:
                    // ```
                    // let chunk_iter = bytes.chunks_exact(CHUNK_SIZE);
                    // let remainder_chunk = chunk_iter.remainder();
                    // for chunk in chunk_iter {
                    //     /* ... */
                    // }
                    // ```
                    // with this:
                    // ```
                    // let chunk_iter = bytes.as_chunks::<CHUNK_SIZE>();
                    // let remainder_chunk = chunk_iter.1;
                    // for chunk in chunk_iter.0.iter() {
                    //     /* ... */
                    // }
                    // ```

                    diag.span_help(call_span, format!("consider using `{as_chunks}` instead"));

                    if let PatKind::Binding(_, _, ident, _) = let_stmt.pat.kind {
                        diag.note(format!(
                            "you can access the chunks using `{ident}.0.iter()`, and the remainder using `{ident}.1`"
                        ));
                    }
                } else {
                    let in_for_loop = {
                        let mut cur_expr = expr;
                        loop {
                            if let Some(parent_expr) = get_parent_expr(cx, cur_expr) {
                                if let Some(for_loop) = ForLoop::hir(parent_expr)
                                    && for_loop.arg.hir_id == expr.hir_id
                                {
                                    break true;
                                }
                                cur_expr = parent_expr;
                            } else {
                                break false;
                            }
                        }
                    };

                    let suffix = if in_for_loop { ".0" } else { ".0.iter()" };
                    diag.span_suggestion(
                        call_span,
                        "consider using `as_chunks` instead",
                        format!("{as_chunks}{suffix}"),
                        applicability,
                    );
                }
            },
        );
    }
}
