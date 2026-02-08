use crate::ast::*;
use crate::token::Token;

// The Parser keeps track of where we are in the list of tokens
// which token we're looking at right now
struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

// Main parse function that starts the parsing process
pub fn parse(tokens: Vec<Token>) -> Result<Program, String> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

// Helpers
impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        return Parser { tokens, current: 0 };
    }

    fn peek(&self) -> Option<&Token> {
        return self.tokens.get(self.current);
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.current);
        self.current += 1;
        return token;
    }

    fn expect(&mut self, expected: Token, error_msg: &str) -> Result<(), String> {
        match self.advance() {
            Some(token) if token == &expected => Ok(()),
            _ => Err(error_msg.to_string()),
        }
    }

    fn precedence(token: &Token) -> Option<u8> {
        match token {
            Token::Star | Token::Slash | Token::Percent => Some(50),
            Token::Plus | Token::Minus => Some(45),
            Token::LessLess | Token::GreaterGreater => Some(40),
            Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual => Some(35),
            Token::EqualEqual | Token::ExclaimEqual => Some(30),
            Token::Ampersand => Some(25),
            Token::Caret => Some(20),
            Token::Pipe => Some(15),
            Token::AmpAmp => Some(10),
            Token::PipePipe => Some(5),
            _ => None,
        }
    }

    fn is_binary_op(token: &Token) -> bool {
        matches!(
            token,
            Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Percent
                | Token::Ampersand
                | Token::Pipe
                | Token::Caret
                | Token::LessLess
                | Token::GreaterGreater
                | Token::AmpAmp
                | Token::PipePipe
                | Token::EqualEqual
                | Token::ExclaimEqual
                | Token::Less
                | Token::LessEqual
                | Token::Greater
                | Token::GreaterEqual
        )
    }

    fn parse_binop(&mut self) -> Result<BinaryOperator, String> {
        match self.advance() {
            Some(Token::Plus) => Ok(BinaryOperator::Add),
            Some(Token::Minus) => Ok(BinaryOperator::Subtract),
            Some(Token::Star) => Ok(BinaryOperator::Multiply),
            Some(Token::Slash) => Ok(BinaryOperator::Divide),
            Some(Token::Percent) => Ok(BinaryOperator::Modulo),
            Some(Token::Ampersand) => Ok(BinaryOperator::BitwiseAnd),
            Some(Token::Pipe) => Ok(BinaryOperator::BitwiseOr),
            Some(Token::Caret) => Ok(BinaryOperator::BitwiseXor),
            Some(Token::LessLess) => Ok(BinaryOperator::LeftShift),
            Some(Token::GreaterGreater) => Ok(BinaryOperator::RightShift),
            Some(Token::AmpAmp) => Ok(BinaryOperator::LogicalAnd),
            Some(Token::PipePipe) => Ok(BinaryOperator::LogicalOr),
            Some(Token::EqualEqual) => Ok(BinaryOperator::Equal),
            Some(Token::ExclaimEqual) => Ok(BinaryOperator::NotEqual),
            Some(Token::Less) => Ok(BinaryOperator::LessThan),
            Some(Token::LessEqual) => Ok(BinaryOperator::LessOrEqual),
            Some(Token::Greater) => Ok(BinaryOperator::GreaterThan),
            Some(Token::GreaterEqual) => Ok(BinaryOperator::GreaterOrEqual),
            _ => Err("Expected binary operator".to_string()),
        }
    }
}

impl Parser {
    fn parse_program(&mut self) -> Result<Program, String> {
        let function = self.parse_function()?;
        self.expect(Token::EOF, "Expected end of file")?;
        return Ok(Program { function });
    }

    fn parse_function(&mut self) -> Result<Function, String> {
        self.expect(Token::Int, "Expected 'int' keyword")?;

        // Expect identifier (function name)
        let name = match self.advance() {
            Some(Token::Identifier(id)) => id.clone(),
            _ => return Err("Expected function name".to_string()),
        };

        self.expect(Token::OpenParen, "Expected '('")?;
        self.expect(Token::Void, "Expected 'void'")?;
        self.expect(Token::CloseParen, "Expected ')'")?;

        self.expect(Token::OpenBrace, "Expected '{'")?;
        let body = self.parse_statement()?;
        self.expect(Token::CloseBrace, "Expected '}'")?;

        return Ok(Function { name, body });
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        self.expect(Token::Return, "Expected 'return' keyword")?;
        let exp = self.parse_exp(0)?;
        self.expect(Token::Semicolon, "Expected ';'")?;
        return Ok(Statement::Return(exp));
    }

    fn parse_exp(&mut self, min_prec: u8) -> Result<Expr, String> {
        let mut left = self.parse_factor()?;

        while let Some(token) = self.peek() {
            // Check if it's a binary operator
            if !Self::is_binary_op(token) {
                break;
            }

            // And Check if its precedence is high enough
            let token_prec = Self::precedence(token).unwrap();
            if token_prec < min_prec {
                break;
            }

            // Parse the operator
            let operator = self.parse_binop()?;

            // Parse the right side with higher precedence
            let right = self.parse_exp(token_prec + 1)?;

            // Build the binary expression
            left = Expr::Binary(operator, Box::new(left), Box::new(right));
        }

        return Ok(left);
    }

    fn parse_factor(&mut self) -> Result<Expr, String> {
        match self.peek() {
            // Constant (Integer)
            Some(Token::Constant(value)) => {
                let value = value.clone();
                self.advance();

                let num = match value.parse::<i32>() {
                    Ok(n) => n,
                    Err(_) => return Err(format!("Invalid number: {}", value)),
                };

                return Ok(Expr::Constant(num));
            }

            // Unary
            Some(Token::Tilde) => {
                self.advance();
                let inner_exp = self.parse_factor()?;
                return Ok(Expr::Unary(UnaryOperator::Complement, Box::new(inner_exp)));
            }

            Some(Token::Minus) => {
                self.advance();
                let inner_exp = self.parse_factor()?;
                return Ok(Expr::Unary(UnaryOperator::Negate, Box::new(inner_exp)));
            }

            Some(Token::Exclaim) => {
                self.advance();
                let inner_exp = self.parse_factor()?;
                return Ok(Expr::Unary(UnaryOperator::LogicalNot, Box::new(inner_exp)));
            }

            // Parenthesized expression "(" <exp> ")"
            Some(Token::OpenParen) => {
                self.advance();
                let inner_exp = self.parse_exp(0)?;

                // Expect closing parenthesis
                match self.advance() {
                    Some(Token::CloseParen) => {}
                    _ => return Err("Expected ')'".to_string()),
                }

                return Ok(inner_exp);
            }

            _ => Err("Expected number, unary operator, or '('".to_string()),
        }
    }
}
