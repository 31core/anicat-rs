use super::token::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct AstNode {
    pub r#type: u8,
    pub data: String,
    pub nodes: Vec<Rc<RefCell<AstNode>>>,
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
    pub fn push(&mut self, node: Rc<RefCell<AstNode>>) {
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
                TOKEN_TYPE_CHAR => new_node.r#type = AST_TYPE_VALUE,
                TOKEN_TYPE_ADD => new_node.r#type = AST_TYPE_ADD, // +
                TOKEN_TYPE_SUB => new_node.r#type = AST_TYPE_SUB, // -
                TOKEN_TYPE_MUL => new_node.r#type = AST_TYPE_MUL, // *
                TOKEN_TYPE_DIV => new_node.r#type = AST_TYPE_DIV, // /
                TOKEN_TYPE_EQU => new_node.r#type = AST_TYPE_VAR_SET_VALUE, // =
                TOKEN_TYPE_LOGIC_AND => new_node.r#type = AST_TYPE_AND, // &&
                TOKEN_TYPE_LOGIC_OR => new_node.r#type = AST_TYPE_OR, // ||
                TOKEN_TYPE_ISEQU => new_node.r#type = AST_TYPE_EQU, // ==
                TOKEN_TYPE_NOTEQU => new_node.r#type = AST_TYPE_NEQU, // !=
                TOKEN_TYPE_LT => new_node.r#type = AST_TYPE_LT,   // <
                TOKEN_TYPE_GT => new_node.r#type = AST_TYPE_GT,   // >
                TOKEN_TYPE_LE => new_node.r#type = AST_TYPE_LE,   // <=
                TOKEN_TYPE_GE => new_node.r#type = AST_TYPE_GE,   // >=
                TOKEN_TYPE_NAME => new_node.r#type = AST_TYPE_IDENTIFIER,
                TOKEN_TYPE_RS_BKT => break,
                TOKEN_TYPE_RM_BKT => break,
                TOKEN_TYPE_RL_BKT => break,
                _ => {}
            }
            /* ( */
            if token.r#type == TOKEN_TYPE_LS_BKT {
                new_node = AstNode::from_tokens(tokens);
                new_node.r#type = AST_TYPE_PARAMS;
            }
            if token.r#type == TOKEN_TYPE_LM_BKT {
                new_node = AstNode::from_tokens(tokens);
                new_node.r#type = AST_TYPE_INDEX;
            }
            /* { */
            else if token.r#type == TOKEN_TYPE_LL_BKT {
                new_node = AstNode::from_tokens(tokens);
                new_node.r#type = AST_TYPE_CODE_BLOCK;
            }
            top_ast.push(Rc::new(RefCell::new(new_node)));
        }
        let mut node_i = 0;
        while node_i < top_ast.nodes.len() {
            /*
               if expression
               elif expression
            */
            if top_ast.nodes[node_i].borrow().r#type == AST_TYPE_IF
                || top_ast.nodes[node_i].borrow().r#type == AST_TYPE_ELIF
            {
                /* add param node */
                let param_node = top_ast.nodes[node_i + 1].clone();
                top_ast.nodes[node_i].borrow_mut().push(param_node);
                top_ast.remove(node_i + 1);

                /* add code block */
                let code_block_node = top_ast.nodes[node_i + 1].clone();
                top_ast.nodes[node_i].borrow_mut().push(code_block_node);
                top_ast.remove(node_i + 1);
            }
            /* function declaration */
            if top_ast.nodes[node_i].borrow().r#type == AST_TYPE_FUNC_DEF {
                /* add identifier node */
                let id_node = top_ast.nodes[node_i + 1].clone();
                top_ast.nodes[node_i].borrow_mut().push(id_node);
                top_ast.remove(node_i + 1);

                /* add param node */
                let param_node = top_ast.nodes[node_i + 1].clone();
                top_ast.nodes[node_i].borrow_mut().push(param_node);
                top_ast.remove(node_i + 1);

                /* function with a return type */
                if top_ast.nodes[node_i + 1].borrow().data == "->" {
                    top_ast.remove(node_i + 1); //remove "->" node
                                                /* add code block */
                    let code_block = top_ast.nodes[node_i + 2].clone();
                    top_ast.nodes[node_i].borrow_mut().push(code_block);

                    /* add ret type */
                    let ret_type = top_ast.nodes[node_i + 1].clone();
                    ret_type.borrow_mut().r#type = AST_TYPE_VAR_TYPE;
                    top_ast.nodes[node_i].borrow_mut().push(ret_type);
                    top_ast.remove(node_i + 1);
                    top_ast.remove(node_i + 1);
                } else {
                    let code_block = top_ast.nodes[node_i + 1].clone();
                    top_ast.nodes[node_i].borrow_mut().push(code_block);
                    top_ast.remove(node_i + 1);
                }
            }
            /* call a function */
            if top_ast.nodes[node_i].borrow().r#type == AST_TYPE_IDENTIFIER
                && node_i < top_ast.nodes.len() - 1
                && top_ast.nodes[node_i + 1].borrow().r#type == AST_TYPE_PARAMS
            {
                let mut func_call_node = AstNode::new();
                func_call_node.r#type = AST_TYPE_FUNC_CALL;
                func_call_node.push(top_ast.nodes[node_i].clone()); //add identifier node
                func_call_node.push(top_ast.nodes[node_i + 1].clone()); //add param node
                top_ast.nodes[node_i] = Rc::new(RefCell::new(func_call_node));
                top_ast.remove(node_i + 1);
            }
            /* declare a variable */
            if top_ast.nodes[node_i].borrow().r#type == AST_TYPE_VAR_DECLARE {
                /* add identifier node */
                let id_node = top_ast.nodes[node_i + 1].clone();
                top_ast.nodes[node_i].borrow_mut().push(id_node);
                top_ast.remove(node_i + 1);
            }
            /* index an array */
            if top_ast.nodes[node_i].borrow().r#type == AST_TYPE_INDEX {
                let index = top_ast.nodes[node_i].borrow().nodes[0].clone();
                let array = top_ast.nodes[node_i - 1].clone();

                top_ast.nodes[node_i].borrow_mut().nodes.pop();
                top_ast.nodes[node_i].borrow_mut().push(array);
                top_ast.nodes[node_i].borrow_mut().push(index);
                top_ast.remove(node_i - 1);
                node_i -= 1;
            }
            if top_ast.nodes[node_i].borrow().data == ":" {
                let type_node = top_ast.nodes[node_i + 1].clone();
                type_node.borrow_mut().r#type = AST_TYPE_VAR_TYPE;
                top_ast.nodes[node_i - 1].borrow_mut().push(type_node);
                top_ast.remove(node_i);
                top_ast.remove(node_i);
                node_i -= 1;
            }
            node_i += 1;
        }
        /* merge '+' '-' '*' '/' nodes */
        let mut node_i = 0;
        while node_i < top_ast.nodes.len() {
            /* operations */
            if top_ast.nodes[node_i].borrow().r#type == AST_TYPE_ADD
                || top_ast.nodes[node_i].borrow().r#type == AST_TYPE_SUB
                || top_ast.nodes[node_i].borrow().r#type == AST_TYPE_MUL
                || top_ast.nodes[node_i].borrow().r#type == AST_TYPE_DIV
            {
                let left = top_ast.nodes[node_i - 1].clone();
                let right = top_ast.nodes[node_i + 1].clone();

                /*
                Last node is ('+' or '-') and this node is ('*' or '/')
                like this:
                    node_i
                      |
                      v
                  +   *
                 / \
                A   B
                */
                if (top_ast.nodes[node_i].borrow().r#type == AST_TYPE_MUL
                    || top_ast.nodes[node_i].borrow().r#type == AST_TYPE_DIV)
                    && (top_ast.nodes[node_i - 1].borrow().r#type == AST_TYPE_ADD
                        || top_ast.nodes[node_i - 1].borrow().r#type == AST_TYPE_SUB)
                {
                    /*
                    *'s right node point to B
                      +   *
                     / \ /
                    A   B
                    */
                    top_ast.nodes[node_i]
                        .borrow_mut()
                        .push(left.borrow().nodes[1].clone());
                    /*
                    *'s right node point to C
                            |\
                      +   * | C
                     / \ / \+
                    A   B
                    */
                    top_ast.nodes[node_i].borrow_mut().push(right);
                    /*
                    +'s right node point to *
                      +
                     / \
                    A   *
                       / \
                      B   C
                    */
                    top_ast.nodes[node_i - 1].borrow_mut().nodes[1] = top_ast.nodes[node_i].clone();
                    top_ast.nodes[node_i] = top_ast.nodes[node_i - 1].clone();
                } else {
                    top_ast.nodes[node_i].borrow_mut().push(left);
                    top_ast.nodes[node_i].borrow_mut().push(right);
                }

                /* remove left and right */
                top_ast.remove(node_i - 1);
                top_ast.remove(node_i);
                node_i -= 1;
            }
            node_i += 1;
        }
        /* merge '==' '!=' '<' '>' '<=' '>=' nodes */
        let mut node_i = 0;
        while node_i < top_ast.nodes.len() {
            if top_ast.nodes[node_i].borrow().r#type == AST_TYPE_EQU
                || top_ast.nodes[node_i].borrow().r#type == AST_TYPE_NEQU
                || top_ast.nodes[node_i].borrow().r#type == AST_TYPE_LT
                || top_ast.nodes[node_i].borrow().r#type == AST_TYPE_GT
                || top_ast.nodes[node_i].borrow().r#type == AST_TYPE_LE
                || top_ast.nodes[node_i].borrow().r#type == AST_TYPE_GE
            {
                let left = top_ast.nodes[node_i - 1].clone();
                let right = top_ast.nodes[node_i + 1].clone();
                top_ast.nodes[node_i].borrow_mut().push(left);
                top_ast.nodes[node_i].borrow_mut().push(right);
                /* remove left and right */
                top_ast.remove(node_i - 1);
                top_ast.remove(node_i);
                node_i -= 1;
            }
            node_i += 1;
        }
        /* merge '&&' '||' nodes */
        let mut node_i = 0;
        while node_i < top_ast.nodes.len() {
            if top_ast.nodes[node_i].borrow().r#type == AST_TYPE_AND
                || top_ast.nodes[node_i].borrow().r#type == AST_TYPE_OR
            {
                let left = top_ast.nodes[node_i - 1].clone();
                let right = top_ast.nodes[node_i + 1].clone();
                top_ast.nodes[node_i].borrow_mut().push(left);
                top_ast.nodes[node_i].borrow_mut().push(right);
                /* remove left and right */
                top_ast.remove(node_i - 1);
                top_ast.remove(node_i);
                node_i -= 1;
            }
            node_i += 1;
        }
        /* handle 'return' '=' node */
        let mut node_i = 0;
        while node_i < top_ast.nodes.len() {
            if top_ast.nodes[node_i].borrow().r#type == AST_TYPE_RETURN {
                let this_node = top_ast.nodes[node_i + 1].clone();
                top_ast.nodes[node_i].borrow_mut().push(this_node);
                top_ast.remove(node_i + 1);
            }
            if top_ast.nodes[node_i].borrow().r#type == AST_TYPE_VAR_SET_VALUE {
                let left = top_ast.nodes[node_i - 1].clone();
                let right = top_ast.nodes[node_i + 1].clone();
                top_ast.nodes[node_i].borrow_mut().push(left);
                top_ast.nodes[node_i].borrow_mut().push(right);
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

pub const AST_TYPE_UNDEFINED: u8 = 0;
pub const AST_TYPE_PROGRAM: u8 = 1;
pub const AST_TYPE_IDENTIFIER: u8 = 2;
pub const AST_TYPE_VAR_DECLARE: u8 = 3;
pub const AST_TYPE_VAR_TYPE: u8 = 4;
pub const AST_TYPE_VAR_SET_VALUE: u8 = 5;
pub const AST_TYPE_FUNC_DEF: u8 = 6;
pub const AST_TYPE_FUNC_CALL: u8 = 7;
pub const AST_TYPE_CODE_BLOCK: u8 = 8;
pub const AST_TYPE_PARAMS: u8 = 9;
pub const AST_TYPE_IF: u8 = 10;
pub const AST_TYPE_ELIF: u8 = 11;
pub const AST_TYPE_ELSE: u8 = 12;
pub const AST_TYPE_FOR: u8 = 13;
pub const AST_TYPE_WHILE: u8 = 14;
pub const AST_TYPE_ADD: u8 = 15; // +
pub const AST_TYPE_SUB: u8 = 16; // -
pub const AST_TYPE_MUL: u8 = 17; // *
pub const AST_TYPE_DIV: u8 = 18; // /
pub const AST_TYPE_GT: u8 = 19; // >
pub const AST_TYPE_LT: u8 = 20; // <
pub const AST_TYPE_GE: u8 = 21; // >=
pub const AST_TYPE_LE: u8 = 22; // >=
pub const AST_TYPE_EQU: u8 = 23; // ==
pub const AST_TYPE_NEQU: u8 = 24; // !=
pub const AST_TYPE_AND: u8 = 25;
pub const AST_TYPE_OR: u8 = 26;
pub const AST_TYPE_VALUE: u8 = 27;
pub const AST_TYPE_BREAK: u8 = 28;
pub const AST_TYPE_CONTINUE: u8 = 29;
pub const AST_TYPE_RETURN: u8 = 30;
pub const AST_TYPE_INDEX: u8 = 31;
