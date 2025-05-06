use pulldown_cmark::{html::push_html, Parser};

pub fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html = String::new();

    push_html(&mut html, parser);

    html
}
