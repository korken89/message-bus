use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod analysis;
mod ast;
mod codegen;

#[proc_macro]
#[proc_macro_error]
pub fn make_message_bus(items: TokenStream) -> TokenStream {
    let ast = match ast::parse(items.into()) {
        Ok(ast) => ast,
        Err(err) => return err.into_compile_error().into(),
    };

    let analysis = match analysis::analyze(&ast) {
        Ok(analysis) => analysis,
        Err(err) => return err.into_compile_error().into(),
    };

    codegen::generate(&ast, &analysis)
}
