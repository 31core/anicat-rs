#[derive(Clone)]
pub struct Token {
    pub r#type: u8,
    pub name: String,
}

pub const TOKEN_TYPE_UNKOWN: u8 = 0;
pub const TOKEN_TYPE_NAME: u8 = 1;
pub const TOKEN_TYPE_KEYWORD: u8 = 2;
pub const TOKEN_TYPE_EQU: u8 = 3; // =
pub const TOKEN_TYPE_EXPLAIN: u8 = 4; // :
pub const TOKEN_TYPE_LS_BKT: u8 = 5; // (
pub const TOKEN_TYPE_LM_BKT: u8 = 6; // [
pub const TOKEN_TYPE_LL_BKT: u8 = 7; // {
pub const TOKEN_TYPE_RS_BKT: u8 = 8; // )
pub const TOKEN_TYPE_RM_BKT: u8 = 9; // ]
pub const TOKEN_TYPE_RL_BKT: u8 = 10; // }
pub const TOKEN_TYPE_ADD: u8 = 11; // +
pub const TOKEN_TYPE_SUB: u8 = 12; // -
pub const TOKEN_TYPE_MUL: u8 = 13; // *
pub const TOKEN_TYPE_DIV: u8 = 14; // /
pub const TOKEN_TYPE_MOD: u8 = 15; // %
pub const TOKEN_TYPE_GT: u8 = 16; // >
pub const TOKEN_TYPE_LT: u8 = 17; // <
pub const TOKEN_TYPE_ISEQU: u8 = 18; // ==
pub const TOKEN_TYPE_NOTEQU: u8 = 19; // !=
pub const TOKEN_TYPE_GE: u8 = 20; // >=
pub const TOKEN_TYPE_LE: u8 = 21; // <=
pub const TOKEN_TYPE_NUMBER: u8 = 22;
pub const TOKEN_TYPE_CHAR: u8 = 23;
pub const TOKEN_TYPE_SPLIT: u8 = 24;
pub const TOKEN_TYPE_STRING: u8 = 25;
pub const TOKEN_TYPE_AND: u8 = 26; // *
pub const TOKEN_TYPE_OR: u8 = 27; // |
pub const TOKEN_TYPE_NOT: u8 = 28; // !
pub const TOKEN_TYPE_LOGIC_AND: u8 = 29; // &&
pub const TOKEN_TYPE_LOGIC_OR: u8 = 30; // ||
pub const TOKEN_TYPE_SHL: u8 = 31; // <<
pub const TOKEN_TYPE_SHR: u8 = 32; // >>
pub const TOKEN_TYPE_DOT: u8 = 33; // .

impl Token {
    pub fn new() -> Self {
        Token {
            r#type: 0,
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
        tokens[i].r#type = TOKEN_TYPE_NAME;
        if is_keyword(&tokens[i].name) {
            tokens[i].r#type = TOKEN_TYPE_KEYWORD;
        } else if is_number(&tokens[i].name) {
            tokens[i].r#type = TOKEN_TYPE_NUMBER;
        }
        /* comment */
        else if tokens[i].name.starts_with("//") || tokens[i].name.starts_with("/*") {
            tokens.remove(i);
            i -= 1;
        } else if tokens[i].name.starts_with("\"") && tokens[i].name.ends_with("\"") {
            tokens[i].r#type = TOKEN_TYPE_STRING;
            /* replace escape characters */
            tokens[i].name = tokens[i].name.replace("\\\"", "\"");
            tokens[i].name = tokens[i].name.replace("\\n", "\n");
            tokens[i].name = tokens[i].name.replace("\\r", "\r");
            tokens[i].name = tokens[i].name.replace("\\t", "\t");
        } else if tokens[i].name.len() == 3
            && tokens[i].name.as_bytes()[0] == '\'' as u8
            && tokens[i].name.as_bytes()[2] == '\'' as u8
        {
            tokens[i].r#type = TOKEN_TYPE_CHAR;
        } else if tokens[i].name == "&" && tokens[i + 1].name == "&" {
            tokens[i].r#type = TOKEN_TYPE_LOGIC_AND;
            tokens[i].name = "&&".to_string();
            tokens.remove(i + 1);
        } else if tokens[i].name == "|" && tokens[i + 1].name == "|" {
            tokens[i].r#type = TOKEN_TYPE_LOGIC_OR;
            tokens[i].name = "||".to_string();
            tokens.remove(i + 1);
        } else if tokens[i].name == "=" && tokens[i + 1].name == "=" {
            tokens[i].r#type = TOKEN_TYPE_ISEQU;
            tokens[i].name = "==".to_string();
            tokens.remove(i + 1);
        } else if tokens[i].name == "!" && tokens[i + 1].name == "=" {
            tokens[i].r#type = TOKEN_TYPE_NOTEQU;
            tokens[i].name = "!=".to_string();
            tokens.remove(i + 1);
        } else if tokens[i].name == "-" && tokens[i + 1].name == ">" {
            tokens[i].r#type = TOKEN_TYPE_EXPLAIN;
            tokens[i].name = "->".to_string();
            tokens.remove(i + 1);
        }
        /* << */
        else if tokens[i].name == "<" && tokens[i + 1].name == "<" {
            tokens[i].name = "<<".to_string();
            tokens[i].r#type = TOKEN_TYPE_SHL;
            tokens.remove(i + 1);
        }
        /* >> */
        else if tokens[i].name == ">" && tokens[i + 1].name == ">" {
            tokens[i].name = ">>".to_string();
            tokens[i].r#type = TOKEN_TYPE_SHR;
            tokens.remove(i + 1);
        }
        /* <, >, <=, >= */
        else if tokens[i].name == ">" || tokens[i].name == "<" {
            /* >= or <= */
            if tokens[i + 1].name == "=" {
                if tokens[i].name == ">" {
                    tokens[i].name = ">=".to_string();
                    tokens[i].r#type = TOKEN_TYPE_GE;
                } else {
                    tokens[i].name = "<=".to_string();
                    tokens[i].r#type = TOKEN_TYPE_LE;
                }

                tokens.remove(i + 1);
            } else if tokens[i].name == ">" {
                tokens[i].r#type = TOKEN_TYPE_GT;
            } else {
                tokens[i].r#type = TOKEN_TYPE_LT;
            }
        } else if tokens[i].name.len() == 1 {
            match &tokens[i].name[..] {
                "&" => tokens[i].r#type = TOKEN_TYPE_AND,
                "|" => tokens[i].r#type = TOKEN_TYPE_OR,
                "!" => tokens[i].r#type = TOKEN_TYPE_NOT,
                "=" => tokens[i].r#type = TOKEN_TYPE_EQU,
                ":" => tokens[i].r#type = TOKEN_TYPE_EXPLAIN,
                "+" => tokens[i].r#type = TOKEN_TYPE_ADD,
                "-" => tokens[i].r#type = TOKEN_TYPE_SUB,
                "*" => tokens[i].r#type = TOKEN_TYPE_MUL,
                "/" => tokens[i].r#type = TOKEN_TYPE_DIV,
                "%" => tokens[i].r#type = TOKEN_TYPE_MOD,
                "(" => tokens[i].r#type = TOKEN_TYPE_LS_BKT,
                "[" => tokens[i].r#type = TOKEN_TYPE_LM_BKT,
                "{" => tokens[i].r#type = TOKEN_TYPE_LL_BKT,
                ")" => tokens[i].r#type = TOKEN_TYPE_RS_BKT,
                "]" => tokens[i].r#type = TOKEN_TYPE_RM_BKT,
                "}" => tokens[i].r#type = TOKEN_TYPE_RL_BKT,
                "," => tokens[i].r#type = TOKEN_TYPE_SPLIT,
                ";" => tokens[i].r#type = TOKEN_TYPE_SPLIT,
                "." => tokens[i].r#type = TOKEN_TYPE_DOT,
                _ => {}
            }
        }
        i += 1;
    }

    Ok(tokens)
}
