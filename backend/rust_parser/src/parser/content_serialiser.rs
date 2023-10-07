use crate::parser;
use crate::parser::Description;

use tl::{NodeHandle, Parser, VDom};

use anyhow::{anyhow, bail, ensure, Context, Result};

// grab some specific htmls and return the html
pub fn grab_htmls(dom: &VDom) -> Result<Description> {
    let parser = dom.parser();
    let content_html = dom
        .get_element_by_id("course-content")
        .context("Unable to find course content")?
        .get(parser)
        .context("Unable to grab parser for the dom, this should not happen")?
        .inner_html(parser);

    let learning_outcome_html = dom
        .get_element_by_id("course-description")
        .context("Unable to find learning outcomes")?
        .get(parser)
        .context("Unable to grab parser for the dom, this should not happen")?
        .inner_html(parser);

    // Handle that recommended qualifications might be none
    let recommended_qualifications_html = dom
        .get_element_by_id("course-skills") // this might be none
        .and_then(|elem| {
            return Some(
                elem.get(parser)
                    .context("Unable to grab parser for the dom, this should not happen")
                    .ok()?
                    .inner_html(parser),
            );
        });

    let recommended_qualifications_html = match recommended_qualifications_html {
        Some(s) => {
            if s.contains("Ingen") || s.contains("None") {
                None
            } else {
                Some(s)
            }
        }
        None => None,
    };

    Ok(Description {
        content: content_html.to_string(),
        learning_outcome: learning_outcome_html.to_string(),
        recommended_qualifications: recommended_qualifications_html.map(|s| s.to_string()),
    })
}
