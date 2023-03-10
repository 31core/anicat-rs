#[derive(Clone)]
pub struct Token {
    pub r#type: TokenType,
    pub name: String,
}

#[derive(Clone, PartialEq, Debug)]
pub enum TokenType {
    Unkown,
    Name,
    Keyword,
    Equ,     // =
    Explain, // :
    LsBkt,   // (
    LmBkt,   // [
    LlBkt,   // {
    RsBkt,   // )
    RmBkt,   // ]
    RlBkt,   // }
    Add,     // +
    Sub,     // -
    Mul,     // *
    Div,     // /
    Mod,     // %
    GT,      // >
    LT,      // <
    IsEqu,   // ==
    NotEqu,  // !=
    Ge,      // >=
    Le,      // <=
    Number,
    Char,
    Split,
    String,
    And,      // *
    Or,       // |
    Not,      // !
    LogicAnd, // &&
    LogicOr,  // ||
    Shl,      // <<
    Shr,      // >>
    Dot,      // .
}

impl Token {
    pub fn new() -> Self {
        Token {
            r#type: TokenType::Unkown,
            name: String::new(),
        }
    }
}

pub const KEYWORDS: [&str; 14] = [
    "var", "func", "return", "if", "elif", "else", "for", "while", "break", "continue", "import",
    "true", "false", "null",
];

/// detect the positions of symbols
fn get_flag_pos(str: &str) -> Result<Vec<isize>, &str> {
    let mut ret: Vec<isize> = vec![];
    ret.push(-1);
    pub const SYMBOLS: &str = " \"\\=()[]{}<>,.:;+-*/&|!\t\n";
    let mut in_string = false;
    let mut in_single_line_comment = false;
    let mut in_multiple_line_comment = false;
    let mut last_char = ' ';
    for i in 0..str.len() {
        for sym in SYMBOLS.chars() {
            if str.as_bytes()[i] == sym as u8 {
                if sym == '"' && last_char != '\\' {
                    in_string = !in_string;
                    continue;
                }

                /* start of single line comment */
                if sym == '/' && last_char == '/' {
                    in_single_line_comment = true;
                    ret.pop();
                    continue;
                }
                /* end of single line comment */
                else if in_single_line_comment && sym == '\n' {
                    in_single_line_comment = false;
                }

                /* start of single line comment */
                if last_char == '/' && sym == '*' {
                    in_multiple_line_comment = true;
                    ret.pop();
                    continue;
                }
                /* end of single line comment */
                else if in_multiple_line_comment && last_char == '*' && sym == '/' {
                    in_multiple_line_comment = false;
                    ret.pop();
                    continue;
                }
                /* if in a string, don't put an in-string synbol into the 'ret' list */
                if !in_string && !in_single_line_comment && !in_multiple_line_comment {
                    ret.push(i as isize);
                }

                last_char = sym;
                break;
            }
        }
    }
    ret.push(str.len() as isize);
    if in_string {
        return Err("sybmol '\"' doesn't match.");
    }
    Ok(ret)
}

/// detect if a keyword
fn is_keyword(str: &str) -> bool {
    for i in 0..KEYWORDS.len() {
        if str == KEYWORDS[i] {
            return true;
        }
    }
    false
}

/// detect if a number
fn is_number(str: &str) -> bool {
    for i in str.chars() {
        if !i.is_numeric() {
            return false;
        }
    }
    true
}

/**
 Generate tokens
*/
pub fn generate_token(code: &str) -> Result<Vec<Token>, &str> {
    let mut tokens: Vec<Token> = vec![];
    let symbol_list;
    match get_flag_pos(code) {
        Ok(lis) => symbol_list = lis,
        Err(err) => return Err(err),
    }
    let code = format!("{code} ");
    for i in 1..symbol_list.len() {
        /* single byte symbol */
        if symbol_list[i] - symbol_list[i - 1] == 1 {
            let mut new_tokens = Token::new();
            new_tokens.name =
                code[symbol_list[i] as usize..(symbol_list[i] + 1) as usize].to_string();
            tokens.push(new_tokens);
        } else {
            let mut new_tokens = Token::new();
            new_tokens.name =
                code[(symbol_list[i - 1] + 1) as usize..symbol_list[i] as usize].to_string();
            tokens.push(new_tokens);

            let mut new_tokens = Token::new();
            new_tokens.name =
                code[symbol_list[i] as usize..(symbol_list[i] + 1) as usize].to_string();
            tokens.push(new_tokens);
        }
    }

    /* delete tokens with meaningless names */
    let mut i = 0;
    while i < tokens.len() {
        if tokens[i].name == " "
            || tokens[i].name == "\t"
            || tokens[i].name == "\n"
            || tokens[i].name == "\r"
        {
            tokens.remove(i);
            i -= 1;
        }
        i += 1;
    }

    let mut i = 0;
    /* detect types */
    while i < tokens.len() {
        tokens[i].r#type = TokenType::Name;
        if is_keyword(&tokens[i].name) {
            tokens[i].r#type = TokenType::Keyword;
        } else if is_number(&tokens[i].name) {
            tokens[i].r#type = TokenType::Number;
        }
        /* comment */
        else if tokens[i].name.starts_with("//") || tokens[i].name.starts_with("/*") {
            tokens.remove(i);
            i -= 1;
        } else if tokens[i].name.starts_with("\"") && tokens[i].name.ends_with("\"") {
            tokens[i].r#type = TokenType::String;
            /* replace escape characters */
            tokens[i].name = tokens[i].name.replace("\\\"", "\"");
            tokens[i].name = tokens[i].name.replace("\\n", "\n");
            tokens[i].name = tokens[i].name.replace("\\r", "\r");
            tokens[i].name = tokens[i].name.replace("\\t", "\t");
        } else if tokens[i].name.len() == 3
            && tokens[i].name.as_bytes()[0] == '\'' as u8
            && tokens[i].name.as_bytes()[2] == '\'' as u8
        {
            tokens[i].r#type = TokenType::Char;
        } else if tokens[i].name == "&" && tokens[i + 1].name == "&" {
            tokens[i].r#type = TokenType::LogicAnd;
            tokens[i].name = "&&".to_string();
            tokens.remove(i + 1);
        } else if tokens[i].name == "|" && tokens[i + 1].name == "|" {
            tokens[i].r#type = TokenType::LogicOr;
            tokens[i].name = "||".to_string();
            tokens.remove(i + 1);
        } else if tokens[i].name == "=" && tokens[i + 1].name == "=" {
            tokens[i].r#type = TokenType::IsEqu;
            tokens[i].name = "==".to_string();
            tokens.remove(i + 1);
        } else if tokens[i].name == "!" && tokens[i + 1].name == "=" {
            tokens[i].r#type = TokenType::NotEqu;
            tokens[i].name = "!=".to_string();
            tokens.remove(i + 1);
        } else if tokens[i].name == "-" && tokens[i + 1].name == ">" {
            tokens[i].r#type = TokenType::Explain;
            tokens[i].name = "->".to_string();
            tokens.remove(i + 1);
        }
        /* << */
        else if tokens[i].name == "<" && tokens[i + 1].name == "<" {
            tokens[i].name = "<<".to_string();
            tokens[i].r#type = TokenType::Shl;
            tokens.remove(i + 1);
        }
        /* >> */
        else if tokens[i].name == ">" && tokens[i + 1].name == ">" {
            tokens[i].name = ">>".to_string();
            tokens[i].r#type = TokenType::Shr;
            tokens.remove(i + 1);
        }
        /* <, >, <=, >= */
        else if tokens[i].name == ">" || tokens[i].name == "<" {
            /* >= or <= */
            if tokens[i + 1].name == "=" {
                if tokens[i].name == ">" {
                    tokens[i].name = ">=".to_string();
                    tokens[i].r#type = TokenType::Ge;
                } else {
                    tokens[i].name = "<=".to_string();
                    tokens[i].r#type = TokenType::Le;
                }

                tokens.remove(i + 1);
            } else if tokens[i].name == ">" {
                tokens[i].r#type = TokenType::GT;
            } else {
                tokens[i].r#type = TokenType::LT;
            }
        } else if tokens[i].name.len() == 1 {
            match &tokens[i].name[..] {
                "&" => tokens[i].r#type = TokenType::And,
                "|" => tokens[i].r#type = TokenType::Or,
                "!" => tokens[i].r#type = TokenType::Not,
                "=" => tokens[i].r#type = TokenType::Equ,
                ":" => tokens[i].r#type = TokenType::Explain,
                "+" => tokens[i].r#type = TokenType::Add,
                "-" => tokens[i].r#type = TokenType::Sub,
                "*" => tokens[i].r#type = TokenType::Mul,
                "/" => tokens[i].r#type = TokenType::Div,
                "%" => tokens[i].r#type = TokenType::Mod,
                "(" => tokens[i].r#type = TokenType::LsBkt,
                "[" => tokens[i].r#type = TokenType::LmBkt,
                "{" => tokens[i].r#type = TokenType::LlBkt,
                ")" => tokens[i].r#type = TokenType::RsBkt,
                "]" => tokens[i].r#type = TokenType::RmBkt,
                "}" => tokens[i].r#type = TokenType::RlBkt,
                "," => tokens[i].r#type = TokenType::Split,
                ";" => tokens[i].r#type = TokenType::Split,
                "." => tokens[i].r#type = TokenType::Dot,
                _ => {}
            }
        }
        i += 1;
    }

    Ok(tokens)
}
