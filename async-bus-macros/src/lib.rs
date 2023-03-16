use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod analysis;
mod ast;
mod codegen;

#[proc_macro]
#[proc_macro_error]
pub fn async_bus(items: TokenStream) -> TokenStream {
    println!("items: \"{}\"", items.to_string());

    let ast = match ast::parse(items.into()) {
        Ok(ast) => ast,
        Err(err) => todo!("parse error {:#?}", err),
    };

    let analysis = match analysis::analyze(&ast) {
        Ok(analysis) => analysis,
        Err(err) => todo!("analysis error {:#?}", err),
    };

    codegen::generate(&ast, &analysis)
}
