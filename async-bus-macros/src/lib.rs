use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::Path,
};

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

    let tokens = codegen::generate(&ast, &analysis);

    write_expansion_to_file(&tokens);

    tokens
}

fn write_expansion_to_file(tokens: &TokenStream) {
    // Default output path: <project_dir>/target/
    let out_dir = Path::new("target");

    if !out_dir.exists() {
        return;
    }

    // Try to write the expanded code to disk
    if let Some(out_str) = out_dir.to_str() {
        let token_string = tokens.to_string();
        let _hash_of_codegen = calculate_hash(&token_string);

        fs::write(format!("{out_str}/async-bus-expansion.rs"), token_string).ok();
    }
}

fn calculate_hash<T>(t: &T) -> u64
where
    T: Hash,
{
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
