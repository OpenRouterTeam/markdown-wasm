use lazy_static::lazy_static;
use pulldown_cmark::{Event, LinkType, Options, Parser, Tag, TagEnd};
use pulldown_cmark_escape::{escape_href, escape_html, escape_html_body_text};
use regex_lite::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tsify::Tsify;
use wasm_bindgen::{convert::IntoWasmAbi, describe::WasmDescribe, prelude::*};

#[derive(Tsify, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[tsify(from_wasm_abi)]
pub struct MarkdownRenderOptions {
    #[serde(default)]
    #[tsify(type = "Record<string, string>")]
    pub link_attributes: BTreeMap<String, String>,

    #[serde(default)]
    #[tsify(type = "Record<string, string>")]
    pub external_link_attributes: BTreeMap<String, String>,

    #[serde(default)]
    pub external_link_icon_html: Option<String>,
}

#[wasm_bindgen(js_name = markdownToHtml)]
pub fn markdown_to_html(markdown: &str, options: Option<MarkdownRenderOptions>) -> JsValue {
    let options = options.unwrap_or_default();
    let parser_options = Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_STRIKETHROUGH
        // We don't actually support math with this renderer, but we want to
        // detect it
        | Options::ENABLE_MATH;
    let parser = Parser::new_ext(markdown, parser_options);

    let mut output = String::new();

    let mut iter = FilteringIterator {
        options: &options,
        inner: parser,
        aborted: false,
        link_is_external: false,
    };

    if let Err(_) = pulldown_cmark::html::write_html_fmt(&mut output, &mut iter) {
        return JsValue::null();
    }

    if iter.aborted {
        JsValue::null()
    } else {
        output.into()
    }
}

struct FilteringIterator<'a> {
    options: &'a MarkdownRenderOptions,
    inner: Parser<'a>,
    aborted: bool,
    link_is_external: bool,
}

impl<'a> Iterator for &'_ mut FilteringIterator<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Event<'a>> {
        if self.aborted {
            return None;
        }
        while let Some(event) = self.inner.next() {
            use pulldown_cmark::Event::*;
            match event {
                Start(Tag::Link {
                    link_type: LinkType::Email,
                    ..
                }) => {
                    // Abort on email link so we can fallback to React renderer
                    self.aborted = true;
                    return None;
                }
                Start(Tag::Link {
                    link_type: _,
                    dest_url,
                    title,
                    id: _,
                }) => {
                    let mut output = String::new();
                    if let Err(_) = self.assemble_link_start(&mut output, &*dest_url, &*title) {
                        self.aborted = true;
                        return None;
                    }
                    return Some(Event::InlineHtml(output.into()));
                }
                End(TagEnd::Link) => {
                    let mut output = String::new();
                    if let Err(_) = self.assemble_link_end(&mut output) {
                        self.aborted = true;
                        return None;
                    }
                    return Some(Event::InlineHtml(output.into()));
                }
                Text(text) => {
                    let mut matches = AUTOLINK_REGEX.find_iter(&*text).peekable();
                    if matches.peek().is_some() {
                        let mut output = String::new();
                        let mut last_consumed_index = 0;
                        for m in matches {
                            if let Err(_) = escape_html_body_text(
                                &mut output,
                                &text[last_consumed_index..m.start()],
                            ) {
                                self.aborted = true;
                                return None;
                            }
                            if let Err(_) = self.assemble_link_start(&mut output, m.as_str(), "") {
                                self.aborted = true;
                                return None;
                            }
                            if let Err(_) = escape_html_body_text(&mut output, m.as_str()) {
                                self.aborted = true;
                                return None;
                            }
                            if let Err(_) = self.assemble_link_end(&mut output) {
                                self.aborted = true;
                                return None;
                            }
                            last_consumed_index = m.end();
                        }
                        if let Err(_) =
                            escape_html_body_text(&mut output, &text[last_consumed_index..])
                        {
                            self.aborted = true;
                            return None;
                        }
                        return Some(InlineHtml(output.into()));
                    } else {
                        // No autolinks, passthrough
                        return Some(Text(text));
                    }
                }
                Start(Tag::Image { .. }) => {
                    // Abort on image so we can fallback to React renderer
                    self.aborted = true;
                    return None;
                }
                Start(Tag::CodeBlock(_)) | Code(_) => {
                    // Abort on code block so we can fallback to React renderer
                    self.aborted = true;
                    return None;
                }
                InlineMath(_) | DisplayMath(_) => {
                    // Abort on math so we can fallback to React renderer
                    self.aborted = true;
                    return None;
                }
                Start(Tag::HtmlBlock) | Html(_) | InlineHtml(_) => {
                    // Reject raw HTML
                    self.aborted = true;
                    return None;
                }
                FootnoteReference(_) => {
                    // Don't let footnote reference insert anchors
                    self.aborted = true;
                    return None;
                }
                event => return Some(event),
            }
        }
        None
    }
}

impl FilteringIterator<'_> {
    fn assemble_link_start<O: pulldown_cmark_escape::StrWrite>(
        &mut self,
        output: &mut O,
        dest_url: &str,
        title: &str,
    ) -> Result<(), O::Error>
    where
        O::Error: Default,
    {
        let is_external = dest_url.starts_with("http");

        write!(output, r#"<a href=""#)?;
        if let Err(_) = escape_href(&mut *output, &dest_url) {
            return Err(O::Error::default());
        }
        if !title.is_empty() {
            write!(output, r#"" title=""#)?;
            if let Err(_) = escape_html(&mut *output, &title) {
                return Err(O::Error::default());
            }
        }
        write!(output, r#"""#)?;
        for (k, v) in &self.options.link_attributes {
            self.write_html_attribute(output, k, v)?;
        }
        if is_external {
            for (k, v) in &self.options.external_link_attributes {
                self.write_html_attribute(output, k, v)?;
            }
        }
        write!(output, ">")?;
        self.link_is_external = is_external;
        Ok(())
    }

    fn assemble_link_end<O: pulldown_cmark_escape::StrWrite>(
        &mut self,
        output: &mut O,
    ) -> Result<(), O::Error> {
        if self.link_is_external {
            if let Some(link_icon_html) = &self.options.external_link_icon_html {
                return write!(output, "{}</a>", link_icon_html);
            }
        }
        write!(output, "</a>")
    }

    fn write_html_attribute<O: pulldown_cmark_escape::StrWrite>(
        &mut self,
        output: &mut O,
        k: &str,
        v: &str,
    ) -> Result<(), O::Error> {
        write!(output, " ")?;
        escape_html(&mut *output, k)?;
        write!(output, r#"=""#)?;
        escape_html(&mut *output, v)?;
        write!(output, r#"""#)?;
        Ok(())
    }
}

lazy_static! {
    static ref AUTOLINK_REGEX: Regex = Regex::new(
        r#"https?:\/\/(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)"#
    ).unwrap();
}
