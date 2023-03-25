use crate::{
    analysis::Analysis,
    ast::{Ast, SubTopic, Topic},
};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::Ident;

fn make_topics_enum(name: &Ident, topics: &[Topic], sub_topics: &[SubTopic]) -> TokenStream2 {
    let mut arms = Vec::new();

    for topic in topics {
        let tn = &topic.name;
        let tp = &topic.payload;
        let doc = format!("Type-level definition of the `{tn}` topic");

        arms.push(quote!(
            #[doc = #doc]
            #tn(#tp)
        ));
    }

    for sub_topic in sub_topics {
        let tn = &sub_topic.name;
        let doc = format!("Type-level definition of the `{tn}` sub-topic");

        arms.push(quote!(
            #[doc = #doc]
            #tn(#tn)
        ));
    }

    let doc = format!("Type-level definition of all topics in `{name}`");
    quote!(
        #[doc = #doc]
        #[derive(Clone)]
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
        let topic_capacity = &topic.capacity;

        let doc_topic = format!("Handle to the `{topic_name}` topic.");
        let doc_sub = format!("Subscribe to the `{topic_name}` topic.");
        let doc_pub = format!("Publish to the `{topic_name}` topic.");

        let publish_parent_topics = subtopic_tracker.to_parent_publishes(topic_name);

        tokens.push(quote!(
            #[doc = #doc_topic]
            pub struct #topic_name;

            #[doc(hidden)]
            #[allow(non_upper_case_globals)]
            static #topic_static: ::message_bus::Topic<#topic_payload> = ::message_bus::Topic::new::<#topic_capacity>();

            impl #topic_name {
                #[doc = #doc_sub]
                pub fn subscribe() -> ::message_bus::Subscriber<#topic_payload> {
                    #topic_static.subscribe()
                }

                #[doc = #doc_pub]
                pub fn publish(payload: #topic_payload) {
                    #(#publish_parent_topics)*

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
    let mut tokens = Vec::new();

    for sub_topic in sub_topics {
        subtopic_tracker.add_subtopic(sub_topic.name.clone());

        let topic_enum = make_topics_enum(
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

        let pub_use = if !subtopic_tracker.at_root() {
            quote!(pub use #sub_topic_module::#sub_topic_name;)
        } else {
            quote!()
        };

        let doc_sub = format!("Subscribe to the `{sub_topic_name}` sub-topic.");

        let mut capacity = 0;
        find_total_capacity(sub_topic, &mut capacity);

        tokens.push(quote!(
            #pub_use

            #[doc = #sub_topic_doc1]
            pub mod #sub_topic_module {
                #[doc(hidden)]
                #[allow(non_upper_case_globals)]
                static #sub_topic_static: ::message_bus::Topic<#sub_topic_name> = ::message_bus::Topic::new::<#capacity>();

                #[doc = #sub_topic_doc2]
                #topic_enum

                impl #sub_topic_name {
                    #[doc = #doc_sub]
                    pub fn subscribe() -> ::message_bus::Subscriber<#sub_topic_name> {
                        #sub_topic_static.subscribe()
                    }
                }

                #(#topics)*

                #(#sub_topic_tokens)*
            }
        ));
        subtopic_tracker.remove_last_subtopic();
    }

    tokens
}

fn find_total_capacity(sub_topic: &SubTopic, capacity: &mut usize) {
    let topic_cap: usize = sub_topic
        .ast
        .topics
        .iter()
        .map(|topic| topic.capacity)
        .sum();

    *capacity += topic_cap;

    for st in &sub_topic.ast.sub_topics {
        find_total_capacity(st, capacity);
    }
}

struct SubTopicTracker(Vec<Ident>);

impl SubTopicTracker {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn depth(&self) -> usize {
        self.0.len()
    }

    pub fn at_root(&self) -> bool {
        self.depth() < 2
    }

    pub fn add_subtopic(&mut self, subtopic: Ident) {
        self.0.push(subtopic);
    }

    pub fn remove_last_subtopic(&mut self) {
        self.0.pop();
    }

    pub fn to_parent_publishes(&self, current_topic: &Ident) -> Vec<TokenStream2> {
        let mut publish_tokens = Vec::new();

        let mut super_tokens = quote!();
        let mut payload = quote!(payload.clone());
        let mut last_topic = current_topic;

        for parent_topic in self.0.iter().rev() {
            let parent_topic_static =
                Ident::new(&format!("__TOPIC_{parent_topic}"), Span::call_site());

            payload = quote!(#super_tokens #parent_topic::#last_topic(#payload));

            publish_tokens.push(quote!(
                #super_tokens #parent_topic_static.publish(#payload);
            ));

            super_tokens = quote!(#super_tokens super::);
            last_topic = parent_topic;
        }

        publish_tokens
    }
}

pub fn generate(ast: &Ast, _anaysis: &Analysis) -> proc_macro::TokenStream {
    let mut subtopic_tracker = SubTopicTracker::new();

    let tokens = codegen_subtopics(&ast.sub_topics, &mut subtopic_tracker);

    quote! {
        #(#tokens)*
    }
    .into()
}
