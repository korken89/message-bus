use crate::{
    analysis::Analysis,
    ast::{Ast, SubTopic, Topic},
};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::Ident;

fn topics_enum(name: &Ident, topics: &[Topic], sub_topics: &[SubTopic]) -> TokenStream2 {
    let mut arms = Vec::new();

    for topic in topics {
        let tn = &topic.name;
        let tp = &topic.payload;

        arms.push(quote!(#tn(#tp)));
    }

    for sub_topic in sub_topics {
        let tn = &sub_topic.name;

        arms.push(quote!(#tn(#tn)));
    }

    quote!(
        pub enum #name {
            #(#arms),*
        }
    )
}

fn codegen_topics(topics: &[Topic]) -> Vec<TokenStream2> {
    let mut tokens = Vec::new();

    for topic in topics {
        let topic_name = &topic.name;
        let topic_payload = &topic.payload;
        let topic_static = Ident::new(&format!("__TOPIC_{topic_name}"), Span::call_site());

        let doc_topic = format!("Handle to the `{topic_name}` topic.");
        let doc_sub = format!("Subscribe to the `{topic_name}` topic.");
        let doc_pub = format!("Publish to the `{topic_name}` topic.");

        tokens.push(quote!(
            #[doc = #doc_topic]
            pub struct #topic_name;

            #[doc(hidden)]
            #[allow(non_upper_case_globals)]
            static #topic_static: ::async_bus::Topic<#topic_payload> = ::async_bus::Topic::new();

            impl #topic_name {
                #[doc = #doc_sub]
                pub fn subscribe() -> ::async_bus::Subscriber<#topic_payload> {
                    #topic_static.subscribe()
                }

                #[doc = #doc_pub]
                pub fn publish(payload: #topic_payload) {
                    #topic_static.publish(payload);
                }
            }
        ));
    }

    tokens
}

fn codegen_subtopics(sub_topics: &[SubTopic]) -> Vec<TokenStream2> {
    let mut tokens = Vec::new();

    for sub_topic in sub_topics {
        let topic_enum = topics_enum(
            &sub_topic.name,
            &sub_topic.ast.topics,
            &sub_topic.ast.sub_topics,
        );
        let topics = codegen_topics(&sub_topic.ast.topics);

        // For the next sub topic, recurse down the tree until bottom is reached
        let sub_topic_tokens = codegen_subtopics(&sub_topic.ast.sub_topics);
        let sub_topic_name = &sub_topic.name;
        let sub_topic_module = &sub_topic.module;
        let sub_topic_doc = format!("");
        let sub_topic_static = Ident::new(&format!("__TOPIC_{sub_topic_name}"), Span::call_site());

        tokens.push(quote!(
            pub use #sub_topic_module::#sub_topic_name;

            pub mod #sub_topic_module {
                #[doc(hidden)]
                #[allow(non_upper_case_globals)]
                static #sub_topic_static: ::async_bus::Topic<#sub_topic_name> = ::async_bus::Topic::new();

                #topic_enum

                #(#topics)*

                #(#sub_topic_tokens)*
            }
            // ..
        ));

        // TODO
    }

    tokens
}

pub fn generate(ast: &Ast, _anaysis: &Analysis) -> proc_macro::TokenStream {
    let toplevel_enum = topics_enum(
        &Ident::new("Toplevel", Span::call_site()),
        &ast.topics,
        &ast.sub_topics,
    );
    let toplevel_topics = codegen_topics(&ast.topics);
    let toplevel_subtopics = codegen_subtopics(&ast.sub_topics);
    let toplevel_static = Ident::new(&format!("__TOPIC_Toplevel"), Span::call_site());

    quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        static #toplevel_static: ::async_bus::Topic<Toplevel> = ::async_bus::Topic::new();

        #toplevel_enum

        #(#toplevel_topics)*

        #(#toplevel_subtopics)*
    }
    .into()
}
