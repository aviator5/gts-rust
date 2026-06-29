#![feature(rustc_private)]

dylint_linting::dylint_library!();

extern crate rustc_ast;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_session;
extern crate rustc_span;

use rustc_ast::LitKind;
use rustc_hir::ExprKind;
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_session::declare_lint;

declare_lint! {
    /// Lint that flags hard-coded GTS identifier prefixes in string literals.
    ///
    /// String literals starting with a configured prefix (default: `"gts."`)
    /// should use the `GTS_ID_PREFIX` constant from the `gts-id` crate, so that
    /// the prefix remains configurable via the `GTS_ID_PREFIX` environment variable.
    ///
    /// For constructing GTS IDs at compile time, use the `gts_id!` macro from
    /// the `gts-macros` crate, which automatically applies the configured prefix.
    ///
    /// The set of flagged prefixes can be customized at lint-load time via the
    /// `GTS_DYLINT_PREFIXES` environment variable (comma-separated, e.g.
    /// `GTS_DYLINT_PREFIXES="gts.,acme."`). Defaults to `gts.`.
    ///
    /// To suppress this lint in specific cases (e.g. constant definitions or
    /// test data), use `#[allow(gts_id_hardcoded_prefix)]`.
    pub GTS_ID_HARDCODED_PREFIX,
    Warn,
    "hard-coded GTS ID prefix in string literal — use GTS_ID_PREFIX constant or the gts_id! macro instead"
}

rustc_session::declare_lint_pass!(GtsIdHardcodedPrefix => [GTS_ID_HARDCODED_PREFIX]);

/// Returns the list of prefixes to flag, read from the `GTS_DYLINT_PREFIXES`
/// environment variable (comma-separated). Defaults to `["gts."]`.
fn configured_prefixes() -> Vec<String> {
    match std::env::var("GTS_DYLINT_PREFIXES") {
        Ok(v) if !v.trim().is_empty() => {
            v.split(',').map(|s| s.trim().to_owned()).filter(|s| !s.is_empty()).collect()
        }
        _ => vec!["gts.".to_owned()],
    }
}

static PREFIXES: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();

fn get_prefixes() -> &'static [String] {
    PREFIXES.get_or_init(configured_prefixes)
}

#[unsafe(no_mangle)]
pub fn register_lints(_sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    lint_store.register_lints(&[GTS_ID_HARDCODED_PREFIX]);
    lint_store.register_late_pass(|_| Box::new(GtsIdHardcodedPrefix));
}

impl<'tcx> LateLintPass<'tcx> for GtsIdHardcodedPrefix {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx rustc_hir::Expr<'tcx>) {
        if let ExprKind::Lit(lit) = &expr.kind
            && let LitKind::Str(symbol, _) = lit.node
        {
            let s = symbol.as_str();
            if !get_prefixes().iter().any(|p| s.starts_with(p.as_str())) {
                return;
            }

            let span = expr.span;
            cx.opt_span_lint(
                GTS_ID_HARDCODED_PREFIX,
                Some(span),
                rustc_errors::DiagDecorator(|diag| {
                    diag.primary_message("hard-coded GTS ID prefix in string literal — use GTS_ID_PREFIX constant or the gts_id! macro instead");
                }),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ui() {
        dylint_testing::ui_test(env!("CARGO_PKG_NAME"), "ui");
    }
}
