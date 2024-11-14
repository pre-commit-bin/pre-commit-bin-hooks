#[test]
fn hook_tests() {
    trycmd::TestCases::new().case("src/*/*.toml");
}

#[cfg(not(windows))]
#[test]
fn readme_config_examples_version_updated() {
    let readme_content = std::fs::read_to_string("README.md").unwrap();
    let expected_version = env!("CARGO_PKG_VERSION");

    let mut new_readme_content = "".to_string();
    for line in readme_content.lines() {
        if line.chars().filter(|c| *c == '@').count() == 1 {
            let split = line.rsplit_once('@').unwrap();
            new_readme_content.push_str(split.0);
            new_readme_content.push('@');
            if split.1.starts_with('v')
                && split.1.chars().skip(1).all(|c| c.is_digit(10) || ['.', '"'].contains(&c) )
            {
                new_readme_content.push('v');
                new_readme_content.push_str(expected_version);
                new_readme_content.push_str(&split.1[split.1.find('"').unwrap()..]);
            } else {
                new_readme_content.push_str(split.1);
            }
        } else {
            new_readme_content.push_str(line);
        }
        new_readme_content.push('\n');
    }
    if readme_content != new_readme_content {
        std::fs::write("README.md", new_readme_content).unwrap();
        assert!(false, "README.md was updated. Run again to pass the test.");
    }
}
