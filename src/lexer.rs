use crate::token::Token;
use regex::Regex;

pub fn lex(source: &str) -> Result<Vec<Token>, String> {
    let mut input = source;
    let mut tokens = Vec::new();

    // Define regexes
    let whitespace = Regex::new(r"^\s+").unwrap();
    let int_kw = Regex::new(r"^int\b").unwrap();
    let void_kw = Regex::new(r"^void\b").unwrap();
    let return_kw = Regex::new(r"^return\b").unwrap();

    let ident = Regex::new(r"^[a-zA-Z_]\w*\b").unwrap();
    let number = Regex::new(r"^[0-9]+\b").unwrap();

    let increment = Regex::new(r"^\+\+").unwrap();
    let decrement = Regex::new(r"^--").unwrap();

    let left_shift = Regex::new(r"^<<").unwrap();
    let right_shift = Regex::new(r"^>>").unwrap();

    let logical_and = Regex::new(r"^&&").unwrap();
    let logical_or = Regex::new(r"^\|\|").unwrap();

    let equal = Regex::new(r"^==").unwrap();
    let not_equal = Regex::new(r"^!=").unwrap();

    let less_equal = Regex::new(r"^<=").unwrap();
    let greater_equal = Regex::new(r"^>=").unwrap();

    while !input.is_empty() {
        // Skip whitespace
        if let Some(m) = whitespace.find(input) {
            input = &input[m.end()..];
            continue;
        }

        // Keywords
        if let Some(m) = int_kw.find(input) {
            tokens.push(Token::Int);
            input = &input[m.end()..];
            continue;
        }
        if let Some(m) = void_kw.find(input) {
            tokens.push(Token::Void);
            input = &input[m.end()..];
            continue;
        }
        if let Some(m) = return_kw.find(input) {
            tokens.push(Token::Return);
            input = &input[m.end()..];
            continue;
        }

        // Identifiers and constants
        if let Some(m) = ident.find(input) {
            tokens.push(Token::Identifier(m.as_str().to_string()));
            input = &input[m.end()..];
            continue;
        }
        if let Some(m) = number.find(input) {
            tokens.push(Token::Constant(m.as_str().to_string()));
            input = &input[m.end()..];
            continue;
        }

        // Operators
        if let Some(m) = left_shift.find(input) {
            tokens.push(Token::LessLess);
            input = &input[m.end()..];
            continue;
        }
        if let Some(m) = right_shift.find(input) {
            tokens.push(Token::GreaterGreater);
            input = &input[m.end()..];
            continue;
        }
        if let Some(m) = logical_and.find(input) {
            tokens.push(Token::AmpAmp);
            input = &input[m.end()..];
            continue;
        }
        if let Some(m) = logical_or.find(input) {
            tokens.push(Token::PipePipe);
            input = &input[m.end()..];
            continue;
        }
        if let Some(m) = equal.find(input) {
            tokens.push(Token::EqualEqual);
            input = &input[m.end()..];
            continue;
        }
        if let Some(m) = not_equal.find(input) {
            tokens.push(Token::ExclaimEqual);
            input = &input[m.end()..];
            continue;
        }
        if let Some(m) = less_equal.find(input) {
            tokens.push(Token::LessEqual);
            input = &input[m.end()..];
            continue;
        }
        if let Some(m) = greater_equal.find(input) {
            tokens.push(Token::GreaterEqual);
            input = &input[m.end()..];
            continue;
        }
        if let Some(m) = increment.find(input) {
            tokens.push(Token::PlusPlus);
            input = &input[m.end()..];
            continue;
        }
        if let Some(m) = decrement.find(input) {
            tokens.push(Token::MinusMinus);
            input = &input[m.end()..];
            continue;
        }

        // Single-character tokens
        let ch = input.chars().next().unwrap();
        match ch {
            '(' => tokens.push(Token::OpenParen),
            ')' => tokens.push(Token::CloseParen),
            '{' => tokens.push(Token::OpenBrace),
            '}' => tokens.push(Token::CloseBrace),
            ';' => tokens.push(Token::Semicolon),
            '~' => tokens.push(Token::Tilde),
            '+' => tokens.push(Token::Plus),
            '-' => tokens.push(Token::Minus),
            '*' => tokens.push(Token::Star),
            '/' => tokens.push(Token::Slash),
            '%' => tokens.push(Token::Percent),
            '&' => tokens.push(Token::Ampersand),
            '|' => tokens.push(Token::Pipe),
            '^' => tokens.push(Token::Caret),
            '!' => tokens.push(Token::Exclaim),
            '<' => tokens.push(Token::Less),
            '>' => tokens.push(Token::Greater),
            '=' => tokens.push(Token::Equal),
            _ => return Err(format!("Unexpected character: '{}'", ch)),
        }
        input = &input[1..];
    }

    tokens.push(Token::EOF);

    return Ok(tokens);
}
