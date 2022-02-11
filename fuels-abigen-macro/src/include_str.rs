

// include_str! : read the given file, insert it as a literal string expr
fn expand_include_str(
    cx: &mut ExtCtxt<'_>,
    sp: Span,
    tts: TokenStream,
    file: &str,
) -> Box<dyn base::MacResult + 'static> {
    // let sp = cx.with_def_site_ctxt(sp);
    // let file = match get_single_str_from_tts(cx, sp, tts, "include_str!") {
    //     Some(f) => f,
    //     None => return DummyResult::any(sp),
    // };
    let file = match resolve_path(file, sp) {
        Ok(f) => f,
        Err(mut err) => {
            err.emit();
            return DummyResult::any(sp);
        }
    };
    match cx.source_map().load_binary_file(&file) {
        Ok(bytes) => match std::str::from_utf8(&bytes) {
            Ok(src) => {
                let interned_src = Symbol::intern(&src);
                base::MacEager::expr(cx.expr_str(sp, interned_src))
            }
            Err(_) => {
                cx.span_err(sp, &format!("{} wasn't a utf-8 file", file.display()));
                DummyResult::any(sp)
            }
        },
        Err(e) => {
            cx.span_err(sp, &format!("couldn't read {}: {}", file.display(), e));
            DummyResult::any(sp)
        }
    }
}

use std::path::PathBuf;
use syntex_errors::diagnostic_builder::DiagnosticBuilder;

/// Resolves a `path` mentioned inside Rust code, returning an absolute path.
///
/// This unifies the logic used for resolving `include_X!`.
fn resolve_path<'a>(
    path: impl Into<PathBuf>,
    span: Span,
) -> Result<PathBuf, DiagnosticBuilder<'a>> {
    let path = path.into();

    // Relative paths are resolved relative to the file in which they are found
    // after macro expansion (that is, they are unhygienic).
    if !path.is_absolute() {
        let callsite = span.source_callsite();
        let mut result = match self.source_map().span_to_filename(callsite) {
            FileName::Real(name) => name
                .into_local_path()
                .expect("attempting to resolve a file path in an external file"),
            FileName::DocTest(path, _) => path,
            other => {
                return Err(self.struct_span_err(
                    span,
                    &format!(
                        "cannot resolve relative path in non-file source `{}`",
                        self.source_map().filename_for_diagnostics(&other)
                    ),
                ));
            }
        };
        result.pop();
        result.push(path);
        Ok(result)
    } else {
        Ok(path)
    }
}