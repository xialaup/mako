use swc_core::common::SyntaxContext;

pub(crate) mod comments;
pub(crate) mod css_ast;
pub(crate) mod error;
pub mod file;
pub(crate) mod js_ast;
pub(crate) mod sourcemap;
#[cfg(test)]
pub mod tests;
pub(crate) mod utils;

pub const DUMMY_CTXT: SyntaxContext = SyntaxContext::empty();
