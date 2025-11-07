use clippy_utils::consts::{ConstEvalCtxt, Constant};
use clippy_utils::diagnostics::span_lint_and_then;
use clippy_utils::msrvs::{self, Msrv};
use clippy_utils::source::snippet_with_applicability;
use clippy_utils::sym;
use rustc_errors::Applicability;
use rustc_hir::Expr;
use rustc_lint::LateContext;
use rustc_span::{Span, Symbol};

use super::CHUNKS_EXACT_WITH_CONST_SIZE;

pub(super) fn check(
    cx: &LateContext<'_>,
    recv: &Expr<'_>,
    arg: &Expr<'_>,
    expr_span: Span,
    call_span: Span,
    method_name: Symbol,
    msrv: Msrv,
) {
    // Check if receiver is slice-like
    if !cx.typeck_results().expr_ty_adjusted(recv).peel_refs().is_slice() {
        return;
    }

    // Check if argument is a constant
    let constant_eval = ConstEvalCtxt::new(cx);
    if let Some(Constant::Int(_)) = constant_eval.eval(arg) {
        // Determine the suggested method name
        let suggestion_method = if method_name == sym::chunks_exact_mut {
            "as_chunks_mut"
        } else {
            "as_chunks"
        };

        // Build the suggestion with proper applicability tracking
        let mut applicability = Applicability::MachineApplicable;
        let recv_str = snippet_with_applicability(cx, recv.span, "_", &mut applicability);
        let arg_str = snippet_with_applicability(cx, arg.span, "_", &mut applicability);

        let suggestion = format!("{recv_str}.{suggestion_method}::<{arg_str}>().0.iter()");

        span_lint_and_then(
            cx,
            CHUNKS_EXACT_WITH_CONST_SIZE,
            call_span,
            format!("using `{method_name}` with a constant chunk size"),
            |diag| {
                diag.span_suggestion(
                    expr_span,
                    "consider using `as_chunks` instead",
                    suggestion,
                    applicability,
                );
            },
        );
    }

    // Check for Rust version
    if !msrv.meets(cx, msrvs::AS_CHUNKS) {}
}
