#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsilTextMethod {
    pub signature: String,
    pub name: String,
    pub start_line: usize,
    pub body: Vec<String>,
}

pub struct MsilParser;

impl MsilParser {
    pub fn parse_methods(source: &str) -> Vec<MsilTextMethod> {
        let lines: Vec<&str> = source.lines().collect();
        parse_methods_from_lines(&lines)
    }
}

fn parse_methods_from_lines(lines: &[&str]) -> Vec<MsilTextMethod> {
    if lines.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut i = 0usize;

    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        if !trimmed.starts_with(".method") {
            i += 1;
            continue;
        }

        let signature = extract_signature(trimmed);
        let (end_line, balanced) = find_block_end(lines, i);
        let end_line = if balanced { end_line } else { lines.len() - 1 };
        let body = lines[i..=end_line.min(lines.len() - 1)].iter().map(|line| (*line).to_string()).collect::<Vec<_>>();

        result.push(MsilTextMethod { name: extract_method_name(&signature), signature, start_line: i + 1, body });

        i = end_line + 1;
    }

    result
}

fn extract_signature(method_line: &str) -> String {
    let sig = method_line[".method".len()..].trim();
    let brace_idx = sig.find('{');
    match brace_idx {
        Some(idx) => sig[..idx].trim(),
        None => sig.trim(),
    }
    .trim_end()
    .to_string()
}

fn extract_method_name(signature: &str) -> String {
    let paren_idx = signature.find('(');
    let head = match paren_idx {
        Some(idx) => signature[..idx].trim(),
        None => signature.trim(),
    };

    let tokens: Vec<&str> = head.split_whitespace().collect();
    match tokens.last() {
        Some(last) => last.to_string(),
        None => signature.to_string(),
    }
}

fn find_block_end(lines: &[&str], method_line_index: usize) -> (usize, bool) {
    let mut depth: i32 = 0;
    let mut seen_open = false;

    for (i, line) in lines.iter().enumerate().skip(method_line_index) {
        for ch in line.chars() {
            if ch == '{' {
                depth += 1;
                seen_open = true;
            }
            else if ch == '}' {
                depth -= 1;
            }

            if seen_open && depth == 0 {
                return (i, true);
            }
        }
    }

    (lines.len() - 1, false)
}
