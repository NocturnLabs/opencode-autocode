use anyhow::{bail, Result};
use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::Reader;

pub fn render_template(template: &str) -> Result<String> {
    let trimmed = template.trim_start();
    if !trimmed.starts_with("<template") {
        return Ok(template.to_string());
    }

    let mut reader = Reader::from_str(template);

    let mut buf = Vec::new();
    let mut content = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(event)) if event.name().as_ref() == b"content" => {
                content = reader.read_text(QName(b"content"))?.into_owned();
                break;
            }
            Ok(Event::Eof) => break,
            Err(err) => return Err(err.into()),
            _ => {}
        }
        buf.clear();
    }

    if content.is_empty() {
        bail!("Template content missing <content> block");
    }

    Ok(content)
}
