use crate::ast::{Ast, SubTopic, Topic};
use std::collections::HashSet;

pub struct Analysis {}

fn check_topics_for_uniqueness(topics: &[Topic], errors: &mut Vec<syn::Error>) {
    let mut uniq = HashSet::new();

    for topic in topics {
        if !uniq.insert(topic.name.clone()) {
            errors.push(syn::Error::new(topic.name.span(), "Non-unique topic name"));
        }
    }
}

fn check_subtopics_for_uniqueness(sub_topics: &[SubTopic], errors: &mut Vec<syn::Error>) {
    let mut uniq = HashSet::new();

    for sub_topic in sub_topics {
        check_topics_for_uniqueness(&sub_topic.ast.topics, errors);

        // For the next sub topic, recurse down the tree until bottom is reached
        check_subtopics_for_uniqueness(&sub_topic.ast.sub_topics, errors);

        if !uniq.insert(sub_topic.name.clone()) {
            errors.push(syn::Error::new_spanned(
                &sub_topic.name,
                "Non-unique sub-topic name",
            ));
        }
    }
}

pub fn analyze(ast: &Ast) -> Result<Analysis, syn::Error> {
    let mut errors = Vec::new();

    // Make sure that the topmost definition defines the toplevel topic
    for topic in &ast.topics {
        errors.push(syn::Error::new_spanned(
            &topic.name,
            "The toplevel definition must be a sub-topic to define the root of the topic tree",
        ));
    }

    if ast.sub_topics.len() > 1 {
        for sub_topic in &ast.sub_topics {
            errors.push(syn::Error::new_spanned(
                &sub_topic.name,
                "Only one toplevel sub-topic is allowed to define the root of the topic tree",
            ));
        }
    }

    // Check for doubly defined topic names in each subtopic level
    check_subtopics_for_uniqueness(&ast.sub_topics, &mut errors);

    // Collect errors if any and return/halt
    if !errors.is_empty() {
        let mut err = errors.get(0).unwrap().clone();
        errors.iter().for_each(|e| err.combine(e.clone()));

        return Err(err);
    }

    Ok(Analysis {})
}
