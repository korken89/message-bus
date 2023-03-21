use proc_macro2::TokenStream as TokenStream2;
use syn::{
    braced, bracketed,
    parse::{self, Error, Parse, ParseStream},
    token, Ident, LitInt, Path, Token,
};

/// Parse a token stream into the AST.
pub fn parse(items: TokenStream2) -> Result<Ast, Error> {
    syn::parse2(items)
}

/// Topic definition `name [optional capacity (usize)] => payload`
#[derive(Debug)]
pub struct Topic {
    pub name: Ident,
    pub payload: Path,
    pub capacity: usize,
}

/// Sub-topic definition `path => { ... }`
#[derive(Debug)]
pub struct SubTopic {
    pub name: Ident,
    pub module: Ident,
    pub ast: Ast,
}

#[derive(Debug)]
pub struct Ast {
    pub topics: Vec<Topic>,
    pub sub_topics: Vec<SubTopic>,
}

fn parse_ast_nodes(input: ParseStream) -> parse::Result<Ast> {
    let mut topics = Vec::new();
    let mut sub_topics = Vec::new();

    loop {
        if input.is_empty() {
            break;
        }

        let path: Path = input.parse()?;

        if path.leading_colon.is_some() {
            return Err(parse::Error::new_spanned(
                &path.leading_colon.unwrap(),
                "Only the forms `Topic` or `sub_topic::SubTopic` is supported, remove the leading colons",
            ));
        }

        for segment in &path.segments {
            if !segment.arguments.is_none() {
                return Err(parse::Error::new_spanned(
                &segment,
                "Only the forms `Topic` or `sub_topic::SubTopic` is supported, remove the generic",
            ));
            }
        }

        let capacity = if input.peek(token::Bracket) {
            let content;
            bracketed!(content in input);

            let lit = content.parse::<LitInt>()?;
            let cap = lit.base10_parse::<usize>()?;

            if cap == 0 {
                return Err(parse::Error::new_spanned(
                    &lit,
                    "Capacity must be larger than 0",
                ));
            }

            cap
        } else {
            1
        };

        let _: Token![=>] = input.parse()?;

        if let Some(ident) = path.get_ident() {
            // Parse a topic 'Topic`

            let name = ident.clone();
            let payload: Path = input.parse()?;

            topics.push(Topic {
                name,
                payload,
                capacity,
            });
        } else if path.segments.len() == 2 {
            // Parse a subtopic 'sub_topic::SubTopic`

            let content;
            braced!(content in input);

            let module = path.segments[0].ident.clone();
            let name = path.segments[1].ident.clone();
            sub_topics.push(SubTopic {
                name,
                module,
                ast: parse_ast_nodes(&content)?,
            });
        } else {
            return Err(parse::Error::new_spanned(
                &path,
                "Only the forms `Topic` or `sub_topic::SubTopic` is supported",
            ));
        }

        // Don't force trailing commas if it's the end of the buffer
        if input.is_empty() {
            break;
        }

        let _: Token![,] = input.parse()?;
    }

    Ok(Ast { topics, sub_topics })
}

impl Parse for Ast {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        let nodes = parse_ast_nodes(input)?;

        Ok(nodes)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ast::parse;
    use quote::{quote, ToTokens};

    fn check_if_topic_exists(node: &Ast, name: &str, payload: &str) -> bool {
        for topic in &node.topics {
            if topic.name.to_string() == name
                && topic.payload.to_token_stream().to_string().replace(" ", "") == payload
            {
                return true;
            }
        }

        for sub_topic in &node.sub_topics {
            if check_if_topic_exists(&sub_topic.ast, name, payload) {
                return true;
            }
        }

        false
    }

    #[test]
    fn prase() {
        let tokens = quote!(
            sub_topic::SubTopic => {
                Foo => i16,
                Bar => u8,
                Bar2 => i8,
                sub_topic2::SubTopic2 => {
                    Baz => i32,
                    Shaz => inner::payload::longer::Value,
                    Fnaz => i128,
                },
            },
            SystemHealth => String, // Primitive payload
            SomeData => some::Data, // Path to payload
            SomeData2 => some::Data2<u32>, // Path to payload
        );

        let ast = parse(tokens).unwrap();

        assert_eq!(ast.topics.len(), 3);
        assert_eq!(ast.sub_topics.len(), 1);
        assert_eq!(ast.sub_topics[0].ast.topics.len(), 3);
        assert_eq!(ast.sub_topics[0].ast.sub_topics.len(), 1);
        assert_eq!(ast.sub_topics[0].ast.sub_topics[0].ast.topics.len(), 3);
        assert_eq!(ast.sub_topics[0].ast.sub_topics[0].ast.sub_topics.len(), 0);

        assert!(!check_if_topic_exists(&ast, "Asdfasdf", "i16"));

        assert!(check_if_topic_exists(&ast, "Foo", "i16"));
        assert!(check_if_topic_exists(&ast, "Bar", "u8"));
        assert!(check_if_topic_exists(&ast, "Bar2", "i8"));
        assert!(check_if_topic_exists(&ast, "Baz", "i32"));
        assert!(check_if_topic_exists(
            &ast,
            "Shaz",
            "inner::payload::longer::Value"
        ));
        assert!(check_if_topic_exists(&ast, "Fnaz", "i128"));
        assert!(check_if_topic_exists(&ast, "SystemHealth", "String"));
        assert!(check_if_topic_exists(&ast, "SomeData", "some::Data"));
        assert!(check_if_topic_exists(&ast, "SomeData2", "some::Data2<u32>"));
    }
}
