use proc_macro2::TokenStream as TokenStream2;
use syn::{parse::Error, Ident, Path};

pub(crate) fn parse(items: TokenStream2) -> Result<Ast, Error> {
    todo!()
}

#[derive(Debug)]
pub struct Topic {
    name: Ident,
    payload: Ident,
}

#[derive(Debug)]
pub enum AstNode {
    Topics(Vec<Topic>),
    SubTopics(Vec<(Path, AstNode)>),
}

#[derive(Debug)]
pub struct Ast {
    base: AstNode,
}

#[cfg(test)]
mod test {
    use crate::ast::parse;
    use quote::quote;

    #[test]
    fn prase() {
        let tokens = quote!(
            sub_topic::SubTopic => {
               Foo => u32,
               Bar => u8,
            },
            SystemHealth => String,
            SomeData => u32,
        );

        let ast = parse(tokens);

        // TODO...
    }
}
