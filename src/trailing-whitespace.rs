use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::process::exit;

/// Unique identifier generator.
///
/// This is not needed for the actual implementation because replacements are
/// done secuentially, but it is used in the tests to generate unique files.
#[cfg(test)]
fn get_id() -> usize {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[cfg(not(test))]
fn get_id() -> usize {
    0
}

fn fix_file(path: &str) -> Result<(), Vec<String>> {
    let maybe_file = OpenOptions::new().read(true).open(path);
    if maybe_file.is_err() {
        return Err(Vec::from([format!(
            "Failed to open {}: {}",
            path,
            maybe_file.unwrap_err().to_string()
        )]));
    }
    let mut file = BufReader::new(maybe_file.unwrap());

    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join(&format!(
        "pre-commit-bin-trailing-whitespace-file{}.txt",
        get_id()
    ));
    if temp_file_path.exists() {
        std::fs::remove_file(&temp_file_path).unwrap();
    }
    let maybe_temp_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&temp_file_path);
    if maybe_temp_file.is_err() {
        return Err(Vec::from([maybe_temp_file.unwrap_err().to_string()]));
    }
    let mut temp_file = maybe_temp_file.unwrap();

    let mut buffer = String::new();
    let mut edited = false;

    loop {
        let bytes_read_result = file.read_line(&mut buffer);
        if bytes_read_result.is_err() {
            return Err(Vec::from([bytes_read_result.unwrap_err().to_string()]));
        }
        if bytes_read_result.unwrap() == 0 {
            break;
        }

        let eof = if buffer.ends_with('\n') {
            buffer.pop();
            if buffer.ends_with('\r') {
                buffer.pop();
                "\r\n"
            } else {
                "\n"
            }
        } else {
            ""
        };

        while buffer.ends_with(' ') || buffer.ends_with('\t') {
            edited = true;
            buffer.pop();
        }

        buffer.push_str(eof);
        temp_file.write_all(buffer.as_bytes()).unwrap();
        buffer.clear();
    }

    if edited {
        std::fs::remove_file(&path).unwrap();
        std::fs::rename(&temp_file_path, &path).unwrap();
    }

    Ok(())
}

fn main() {
    let args = std::env::args();
    if args.len() == 1 {
        eprintln!("No arguments provided");
        exit(1);
    }

    let mut errors: Vec<String> = Vec::new();
    for file_path in args.skip(1) {
        if let Err(errs) = fix_file(&file_path) {
            errors.extend(errs);
        }
    }

    if !errors.is_empty() {
        for err in errors {
            eprintln!("{}", err);
        }
        exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_fix(input: &str, expected: &str) {
        let temp = std::env::temp_dir();
        let file_path = temp.join(format!("test_trailing_whitespace_{}.txt", get_id()));
        if file_path.exists() {
            std::fs::remove_file(&file_path).expect("Failed to remove file");
        }
        std::fs::write(&file_path, input).unwrap();

        let result = fix_file(file_path.to_str().unwrap());
        assert!(
            result.is_ok(),
            "Failed to fix file: {:?}",
            result.unwrap_err()
        );

        let new_content = std::fs::read_to_string(&file_path).expect("Failed to read file");
        std::fs::remove_file(&file_path).expect("Failed to remove file");
        assert_eq!(new_content, expected);
    }

    #[test]
    fn test_fix_whitespaces_at_end() {
        assert_fix("Hello, world!   \n  ", "Hello, world!\n");
    }

    #[test]
    fn test_fix_whitespaces_before_unix_newline() {
        assert_fix("Hello, world!   \n  \n", "Hello, world!\n\n");
    }

    #[test]
    fn test_fix_whitespaces_before_windows_newline() {
        assert_fix("  Hello, world!   \r\n  \r\n", "  Hello, world!\r\n\r\n");
    }
}
