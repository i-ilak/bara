use pulldown_cmark::html;
use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, TagEnd};
use std::collections::HashMap;
use std::fmt::Write;

pub fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_FOOTNOTES);
    let parser = Parser::new_ext(markdown, options);

    let mut context = ParsingContext {
        footnotes: Vec::new(),
        in_footnote: Vec::new(),
        footnote_numbers: HashMap::new(),
        html_output: String::new(),
    };
    let events: Vec<Event> = filter_footnotes(parser, &mut context);

    process_main_content(events.into_iter(), &mut context);
    if !context.footnotes.is_empty() {
        process_footnotes(&mut context);
    }

    context.html_output
}

struct ParsingContext<'a> {
    footnotes: Vec<Vec<Event<'a>>>,
    in_footnote: Vec<Vec<Event<'a>>>,
    footnote_numbers: HashMap<CowStr<'a>, (usize, usize)>,
    html_output: String,
}

fn filter_footnotes<'a>(parser: Parser<'a>, context: &mut ParsingContext<'a>) -> Vec<Event<'a>> {
    parser.filter_map(|event| {
        match event {
            Event::Start(Tag::FootnoteDefinition(_)) => {
                context.in_footnote.push(vec![event]);
                None
            }
            Event::End(TagEnd::FootnoteDefinition) => {
                let mut f = context.in_footnote.pop().unwrap();
                f.push(event);
                context.footnotes.push(f);
                None
            }
            Event::FootnoteReference(name) => {
                let n = context.footnote_numbers.len() + 1;
                let (n, nr) = context.footnote_numbers.entry(name.clone()).or_insert((n, 0usize));
                *nr += 1;
                Some(Event::Html(
                    format!(
                        r##"<sup class="footnote-reference" id="fr-{name}-{nr}"><a href="#fn-{name}">[{n}]</a></sup>"##
                    )
                    .into(),
                ))
            }
            _ if !context.in_footnote.is_empty() => {
                context.in_footnote.last_mut().unwrap().push(event);
                None
            }
            _ => Some(event),
        }
    }).collect()
}

fn process_main_content<'a>(
    events: impl Iterator<Item = Event<'a>>,
    context: &mut ParsingContext<'a>,
) {
    for event in events {
        match event {
            Event::Start(Tag::CodeBlock(_kind)) => {
                handle_code_block_start(context);
            }
            Event::End(TagEnd::CodeBlock) => {
                handle_code_block_end(context);
            }
            Event::Text(text) => {
                context.html_output.push_str(&html_escape::encode_text(&text));
            }
            _ => {
                html::push_html(&mut context.html_output, std::iter::once(event));
            }
        }
    }
}

fn handle_code_block_start(context: &mut ParsingContext<'_>) {
    context.html_output.push_str("<pre><code>");
}

fn handle_code_block_end(context: &mut ParsingContext<'_>) {
    context.html_output.push_str("</code></pre>");
}

fn process_footnotes(context: &mut ParsingContext<'_>) {
    // Retain only footnotes that are referenced at least once
    context.footnotes.retain(|f| match f.first() {
        Some(Event::Start(Tag::FootnoteDefinition(name))) => {
            context.footnote_numbers.get(name).unwrap_or(&(0, 0)).1 != 0
        }
        _ => false,
    });

    // Sort footnotes by their number
    context.footnotes.sort_by_cached_key(|f| match f.first() {
        Some(Event::Start(Tag::FootnoteDefinition(name))) => {
            context.footnote_numbers.get(name).unwrap_or(&(0, 0)).0
        }
        _ => unreachable!(),
    });

    // Begin footnotes section
    context
        .html_output
        .push_str("<hr><ol class=\"footnotes-list\">\n");

    // Process each footnote
    let mut footnotes = std::mem::take(&mut context.footnotes);
    for footnote in footnotes.iter_mut() {
        process_single_footnote(footnote, context);
    }

    context.html_output.push_str("</ol>");
}

fn process_single_footnote<'a>(footnote: &mut Vec<Event<'a>>, context: &mut ParsingContext<'a>) {
    let mut name = CowStr::from("");
    let mut has_written_backrefs = false;
    let fl_len = footnote.len();

    let events = std::mem::take(footnote);
    for (i, event) in events.into_iter().enumerate() {
        match event {
            Event::Start(Tag::FootnoteDefinition(n)) => {
                name = n;
                write!(context.html_output, r##"<li id="fn-{}">"##, name).unwrap();
            }
            Event::End(TagEnd::FootnoteDefinition) | Event::End(TagEnd::Paragraph)
                if !has_written_backrefs && i >= fl_len - 2 =>
            {
                write_footnote_backreferences(&name, context);
                has_written_backrefs = true;
                context.html_output.push_str("</li>\n");
            }
            Event::End(TagEnd::FootnoteDefinition) => {
                context.html_output.push_str("</li>\n");
            }
            e => html::push_html(&mut context.html_output, std::iter::once(e)),
        }
    }
}

fn write_footnote_backreferences(name: &CowStr<'_>, context: &mut ParsingContext<'_>) {
    if let Some((_, count)) = context.footnote_numbers.get(name) {
        for usage in 1..=*count {
            write!(
                context.html_output,
                r##" <a href="#fr-{}-{}">↩{}</a>"##,
                name,
                usage,
                if usage > 1 {
                    usage.to_string()
                } else {
                    String::new()
                }
            )
            .unwrap();
        }
    }
}
