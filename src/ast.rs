use super::token::*;
#[derive(Clone)]
pub struct AstNode {
    pub r#type: u8,
    pub data: String,
    pub nodes: Vec<AstNode>,
}

impl AstNode {
    pub fn new() -> Self {
        AstNode {
            r#type: 0,
            data: String::new(),
            nodes: Vec::new(),
        }
    }
    /// push a subnode
    pub fn push(&mut self, node: &AstNode) {
        self.nodes.push(node.clone());
    }
    /// remove a subnode
    pub fn remove(&mut self, index: usize) {
        self.nodes.remove(index);
    }
    pub fn from_tokens<T>(tokens: &mut T) -> Self
    where
        T: Iterator<Item = Token>,
    {
        let mut top_ast = AstNode::new();
        loop {
            let token;
            match tokens.next() {
                Some(tk) => token = tk,
                None => break,
            }
            let mut new_node = AstNode::new();
            new_node.data = token.name.clone();
            /* keywords */
            if token.r#type == TOKEN_TYPE_KEYWORD {
                match &token.name[..] {
                    "func" => new_node.r#type = AST_TYPE_FUNC_DEF,
                    "var" => new_node.r#type = AST_TYPE_VAR_DECLARE,
                    "if" => new_node.r#type = AST_TYPE_IF,
                    "elif" => new_node.r#type = AST_TYPE_ELIF,
                    "else" => new_node.r#type = AST_TYPE_ELSE,
                    "for" => new_node.r#type = AST_TYPE_FOR,
                    "while" => new_node.r#type = AST_TYPE_WHILE,
                    "break" => new_node.r#type = AST_TYPE_BREAK,
                    "continue" => new_node.r#type = AST_TYPE_CONTINUE,
                    "return" => new_node.r#type = AST_TYPE_RETURN,
                    "true" => new_node.r#type = AST_TYPE_VALUE,
                    "false" => new_node.r#type = AST_TYPE_VALUE,
                    _ => {}
                }
            }
            match token.r#type {
                TOKEN_TYPE_NUMBER => new_node.r#type = AST_TYPE_VALUE,
                TOKEN_TYPE_STRING => new_node.r#type = AST_TYPE_VALUE,
                TOKEN_TYPE_ADD => new_node.r#type = AST_TYPE_ADD,
                TOKEN_TYPE_SUB => new_node.r#type = AST_TYPE_SUB,
                TOKEN_TYPE_MUL => new_node.r#type = AST_TYPE_MUL,
                TOKEN_TYPE_DIV => new_node.r#type = AST_TYPE_DIV,
                TOKEN_TYPE_LOGIC_AND => new_node.r#type = AST_TYPE_AND, // &&
                TOKEN_TYPE_LOGIC_OR => new_node.r#type = AST_TYPE_OR,   // ||
                TOKEN_TYPE_ISEQU => new_node.r#type = AST_TYPE_EQU,     // ==
                TOKEN_TYPE_LT => new_node.r#type = AST_TYPE_LT,         // <
                TOKEN_TYPE_GT => new_node.r#type = AST_TYPE_GT,         // >
                TOKEN_TYPE_LE => new_node.r#type = AST_TYPE_LE,         // <=
                TOKEN_TYPE_GE => new_node.r#type = AST_TYPE_GE,         // >=
                TOKEN_TYPE_NAME => new_node.r#type = AST_TYPE_IDENTIFIER,
                TOKEN_TYPE_RS_BKT => break,
                TOKEN_TYPE_RL_BKT => break,
                _ => {}
            }
            /* ( */
            if token.r#type == TOKEN_TYPE_LS_BKT {
                new_node = AstNode::from_tokens(tokens);
                new_node.r#type = AST_TYPE_PARAMS;
            }
            /* { */
            else if token.r#type == TOKEN_TYPE_LL_BKT {
                new_node = AstNode::from_tokens(tokens);
                new_node.r#type = AST_TYPE_CODE_BLOCK;
            }
            top_ast.push(&new_node);
        }
        let mut node_i = 0;
        while node_i < top_ast.nodes.len() {
            /*
               if expression
               elif expression
            */
            if top_ast.nodes[node_i].r#type == AST_TYPE_IF
                || top_ast.nodes[node_i].r#type == AST_TYPE_ELIF
            {
                /* add param node */
                let param_node = top_ast.nodes[node_i + 1].clone();
                top_ast.nodes[node_i].push(&param_node);
                top_ast.remove(node_i + 1);

                /* add code block */
                let code_block_node = top_ast.nodes[node_i + 1].clone();
                top_ast.nodes[node_i].push(&code_block_node);
                top_ast.remove(node_i + 1);
            }
            /* function declaration */
            if top_ast.nodes[node_i].r#type == AST_TYPE_FUNC_DEF {
                /* add identifier node */
                let id_node = top_ast.nodes[node_i + 1].clone();
                top_ast.nodes[node_i].push(&id_node);
                top_ast.remove(node_i + 1);

                /* add param node */
                let param_node = top_ast.nodes[node_i + 1].clone();
                top_ast.nodes[node_i].push(&param_node);
                top_ast.remove(node_i + 1);

                /* function with a return type */
                if top_ast.nodes[node_i + 1].data == "->" {
                    top_ast.remove(node_i + 1); //remove "->" node
                                                /* add code block */
                    let code_block = top_ast.nodes[node_i + 2].clone();
                    top_ast.nodes[node_i].push(&code_block);

                    /* add ret type */
                    let ret_type = top_ast.nodes[node_i + 1].clone();
                    top_ast.nodes[node_i].push(&ret_type);
                    top_ast.remove(node_i + 1);
                    top_ast.remove(node_i + 1);
                } else {
                    let code_block = &top_ast.nodes[node_i + 1].clone();
                    top_ast.nodes[node_i].push(&code_block);
                    top_ast.remove(node_i + 1);
                }
            }
            /* call a function */
            if top_ast.nodes[node_i].r#type == AST_TYPE_IDENTIFIER
                && node_i < top_ast.nodes.len() - 1
                && top_ast.nodes[node_i + 1].r#type == AST_TYPE_PARAMS
            {
                let mut func_call_node = AstNode::new();
                func_call_node.r#type = AST_TYPE_FUNC_CALL;
                func_call_node.push(&top_ast.nodes[node_i]); //add identifier node
                func_call_node.push(&top_ast.nodes[node_i + 1]); //add param node
                top_ast.nodes[node_i] = func_call_node;
                top_ast.remove(node_i + 1);
            }
            /* declare a variable */
            if top_ast.nodes[node_i].r#type == AST_TYPE_VAR_DECLARE {
                /* add identifier node */
                let id_node = top_ast.nodes[node_i + 1].clone();
                top_ast.nodes[node_i].push(&id_node);
                top_ast.remove(node_i + 1);
            }
            /* operations */
            if top_ast.nodes[node_i].r#type == AST_TYPE_ADD
                || top_ast.nodes[node_i].r#type == AST_TYPE_SUB
                || top_ast.nodes[node_i].r#type == AST_TYPE_MUL
                || top_ast.nodes[node_i].r#type == AST_TYPE_DIV
            {
                let left = top_ast.nodes[node_i - 1].clone();
                let right = top_ast.nodes[node_i + 1].clone();

                /*
                Last node is ('+' or '-') and this node is ('*' or '/')
                like this:
                  +    *    C
                 / \
                A   B
                */
                if (top_ast.nodes[node_i].r#type == AST_TYPE_MUL
                    || top_ast.nodes[node_i].r#type == AST_TYPE_DIV)
                    && (top_ast.nodes[node_i - 1].r#type == AST_TYPE_ADD
                        || top_ast.nodes[node_i - 1].r#type == AST_TYPE_SUB)
                {
                    top_ast.nodes[node_i].push(&left.nodes[1]);
                    top_ast.nodes[node_i].push(&right);
                    top_ast.nodes[node_i - 1].nodes[1] = top_ast.nodes[node_i].clone();
                    top_ast.nodes[node_i] = top_ast.nodes[node_i - 1].clone();
                } else {
                    top_ast.nodes[node_i].push(&left);
                    top_ast.nodes[node_i].push(&right);
                }

                /* remove left and right */
                top_ast.remove(node_i - 1);
                top_ast.remove(node_i);
                node_i -= 1;
            }
            node_i += 1;
        }
        top_ast
    }
}

#[allow(dead_code)]
const AST_TYPE_UNDEFINED: u8 = 0;
#[allow(dead_code)]
const AST_TYPE_PROGRAM: u8 = 1;
const AST_TYPE_IDENTIFIER: u8 = 2;
const AST_TYPE_VAR_DECLARE: u8 = 3;
#[allow(dead_code)]
const AST_TYPE_VAR_TYPE: u8 = 4;
#[allow(dead_code)]
const AST_TYPE_VAR_SET_VALUE: u8 = 5;
#[allow(dead_code)]
const AST_TYPE_VAR_GET_VALUE: u8 = 6;
const AST_TYPE_FUNC_DEF: u8 = 7;
const AST_TYPE_FUNC_CALL: u8 = 8;
const AST_TYPE_CODE_BLOCK: u8 = 9;
const AST_TYPE_PARAMS: u8 = 10;
const AST_TYPE_IF: u8 = 11;
const AST_TYPE_ELIF: u8 = 12;
const AST_TYPE_ELSE: u8 = 13;
const AST_TYPE_FOR: u8 = 14;
const AST_TYPE_WHILE: u8 = 15;
const AST_TYPE_ADD: u8 = 16; // +
const AST_TYPE_SUB: u8 = 17; // -
const AST_TYPE_MUL: u8 = 18; // *
const AST_TYPE_DIV: u8 = 19; // /
const AST_TYPE_GT: u8 = 20; // >
const AST_TYPE_LT: u8 = 21; // <
const AST_TYPE_GE: u8 = 22; // >=
const AST_TYPE_LE: u8 = 23; // >=
const AST_TYPE_EQU: u8 = 24; // ==
const AST_TYPE_AND: u8 = 25;
const AST_TYPE_OR: u8 = 26;
const AST_TYPE_VALUE: u8 = 27;
const AST_TYPE_BREAK: u8 = 28;
const AST_TYPE_CONTINUE: u8 = 29;
const AST_TYPE_RETURN: u8 = 30;
