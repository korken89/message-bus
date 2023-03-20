use crate::{
    analysis::Analysis,
    ast::{Ast, SubTopic, Topic},
};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, TokenStreamExt};
use std::collections::HashMap;
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

fn codegen_topics(topics: &[Topic], subtopic_tracker: &mut SubTopicTracker) -> Vec<TokenStream2> {
    let mut tokens = Vec::new();

    for topic in topics {
        let topic_name = &topic.name;
        let topic_payload = &topic.payload;
        let topic_static = Ident::new(&format!("__TOPIC_{topic_name}"), Span::call_site());

        let doc_topic = format!("Handle to the `{topic_name}` topic.");
        let doc_sub = format!("Subscribe to the `{topic_name}` topic.");
        let doc_pub = format!("Publish to the `{topic_name}` topic.");

        let mut publish_parent_topic = Vec::new();

        for (parent_topic, depth) in &subtopic_tracker.0 {
            let parent_topic_static =
                Ident::new(&format!("__TOPIC_{parent_topic}"), Span::call_site());
            let mut super_tokens = TokenStream2::new();

            for _ in 0..*depth {
                super_tokens.extend(quote!(super::));
            }

            // if *depth > 0 {
            //     super_tokens.extend(quote!(::));
            // }

            // ..
            publish_parent_topic.push(quote!(
                #super_tokens #parent_topic_static.publish(payload.clone());
            ));
        }

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
                    #(#publish_parent_topic)*
                    /// TEST
                    #topic_static.publish(payload);
                }
            }
        ));
    }

    tokens
}

fn codegen_subtopics(
    sub_topics: &[SubTopic],
    subtopic_tracker: &mut SubTopicTracker,
) -> Vec<TokenStream2> {
    subtopic_tracker.increase_depth();

    let mut tokens = Vec::new();

    for sub_topic in sub_topics {
        subtopic_tracker.add_subtopic(sub_topic.name.clone());

        let topic_enum = topics_enum(
            &sub_topic.name,
            &sub_topic.ast.topics,
            &sub_topic.ast.sub_topics,
        );
        let topics = codegen_topics(&sub_topic.ast.topics, subtopic_tracker);

        // For the next sub topic, recurse down the tree until bottom is reached
        let sub_topic_tokens = codegen_subtopics(&sub_topic.ast.sub_topics, subtopic_tracker);
        let sub_topic_name = &sub_topic.name;
        let sub_topic_module = &sub_topic.module;
        let sub_topic_doc1 = format!(
            "Module containing topics and implementation for the `{sub_topic_name}` subtopic"
        );
        let sub_topic_doc2 =
            format!("All topics in the `{sub_topic_module}::{sub_topic_name}` subtopic");
        let sub_topic_static = Ident::new(&format!("__TOPIC_{sub_topic_name}"), Span::call_site());

        tokens.push(quote!(
            pub use #sub_topic_module::#sub_topic_name;

            #[doc = #sub_topic_doc1]
            pub mod #sub_topic_module {
                #[doc(hidden)]
                #[allow(non_upper_case_globals)]
                static #sub_topic_static: ::async_bus::Topic<#sub_topic_name> = ::async_bus::Topic::new();

                #[doc = #sub_topic_doc2]
                #topic_enum

                #(#topics)*

                #(#sub_topic_tokens)*
            }
        ));
    }

    subtopic_tracker.decrease_depth();

    tokens
}

struct SubTopicTracker(HashMap<Ident, usize>);

impl SubTopicTracker {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_subtopic(&mut self, subtopic: Ident) {
        println!("Adding subtopic: {subtopic:?}");
        assert!(self.0.insert(subtopic, 0).is_none());
    }

    pub fn increase_depth(&mut self) {
        println!("Increasing depth, pre: {:#?}", self.0);
        for (_, v) in &mut self.0 {
            *v += 1;
        }
        println!("Increasing depth, post: {:#?}", self.0);
    }

    pub fn decrease_depth(&mut self) {
        let mut to_remove = vec![];

        println!("Decreasing depth, pre: {:#?}", self.0);
        for (k, v) in &mut self.0 {
            if *v == 0 {
                to_remove.push(k.clone());
            } else {
                *v -= 1;
            }
        }

        for t in &to_remove {
            self.0.remove(t);
        }
        println!("Decreasing depth, post: {:#?}", self.0);
    }
}

pub fn generate(ast: &Ast, _anaysis: &Analysis) -> proc_macro::TokenStream {
    let toplevel_ident = Ident::new("Toplevel", Span::call_site());
    let toplevel_enum = topics_enum(&toplevel_ident, &ast.topics, &ast.sub_topics);

    let mut subtopic_tracker = SubTopicTracker::new();

    subtopic_tracker.add_subtopic(toplevel_ident);

    let toplevel_topics = codegen_topics(&ast.topics, &mut subtopic_tracker);
    let toplevel_subtopics = codegen_subtopics(&ast.sub_topics, &mut subtopic_tracker);
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
