
#[cfg(test)]
mod tests {
    use dialoguer::Editor;

    #[test]
    fn test_editor_exists() {
        let _ = Editor::new();
    }
}
