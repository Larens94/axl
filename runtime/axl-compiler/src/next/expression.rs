use std::collections::BTreeMap;

use serde_json::{Number, Value};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Path(Vec<String>),
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Conditional {
        condition: Box<Expr>,
        when_true: Box<Expr>,
        when_false: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Negate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Or,
    And,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Ident(String),
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    If,
    Then,
    Else,
    Or,
    And,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Plus,
    Minus,
    Star,
    Slash,
    Bang,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Comma,
    End,
}

pub fn parse(source: &str) -> Result<Expr, String> {
    let tokens = tokenize(source)?;
    let mut parser = Parser { tokens, cursor: 0 };
    let expression = parser.parse_conditional()?;
    if parser.peek() != &Token::End {
        return Err("unexpected token after expression".into());
    }
    Ok(expression)
}

pub fn evaluate(expression: &Expr, values: &BTreeMap<String, Value>) -> Result<Value, String> {
    match expression {
        Expr::Path(path) => {
            if is_uuid_v4_builtin(path) {
                return Ok(Value::String(uuid::Uuid::new_v4().to_string()));
            }
            if path.first().is_some_and(|first| first == "uuid") {
                return Err(format!(
                    "unknown uuid builtin '{}'",
                    path.get(1).map(String::as_str).unwrap_or("")
                ));
            }
            let mut value = values
                .get(&path[0])
                .ok_or_else(|| format!("unknown runtime value '{}'", path[0]))?;
            for segment in &path[1..] {
                match value {
                    // A declared-but-absent optional field reads as `null`. Field
                    // access is validated by the analyzer, so a missing member can
                    // only be an optional that was not provided.
                    Value::Object(object) => match object.get(segment) {
                        Some(next) => value = next,
                        None => return Ok(Value::Null),
                    },
                    // Reading through an absent optional stays absent.
                    Value::Null => return Ok(Value::Null),
                    _ => return Err(format!("value path '{}' is missing", path.join("."))),
                }
            }
            Ok(value.clone())
        }
        Expr::Bool(value) => Ok(Value::Bool(*value)),
        Expr::Int(value) => Ok(Value::Number(Number::from(*value))),
        Expr::Float(value) => Number::from_f64(*value)
            .map(Value::Number)
            .ok_or_else(|| "floating-point result is not finite".into()),
        Expr::String(value) => Ok(Value::String(value.clone())),
        Expr::List(items) => items
            .iter()
            .map(|item| evaluate(item, values))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        Expr::Unary(operator, value) => {
            let value = evaluate(value, values)?;
            match operator {
                UnaryOp::Not => value
                    .as_bool()
                    .map(|value| Value::Bool(!value))
                    .ok_or_else(|| "operator ! requires bool".into()),
                UnaryOp::Negate => {
                    if let Some(value) = value.as_i64() {
                        return value
                            .checked_neg()
                            .map(|value| Value::Number(Number::from(value)))
                            .ok_or_else(|| "integer negation overflow".into());
                    }
                    number_value(-number(&value)?)
                }
            }
        }
        Expr::Binary(left, operator, right) => {
            let left = evaluate(left, values)?;
            let right = evaluate(right, values)?;
            evaluate_binary(left, *operator, right)
        }
        Expr::Conditional {
            condition,
            when_true,
            when_false,
        } => {
            if boolean(&evaluate(condition, values)?)? {
                evaluate(when_true, values)
            } else {
                evaluate(when_false, values)
            }
        }
    }
}

/// Evaluate an expression that feeds an optional target.
///
/// Returns `Ok(None)` when the expression evaluates to an absent value (`null`),
/// which lets `make` leave an optional field absent instead of storing a `null`
/// that would fail entity validation. Genuine evaluation errors (type
/// mismatches, arithmetic on non-numbers, unknown builtins, …) still propagate.
pub fn evaluate_optional(
    expression: &Expr,
    values: &BTreeMap<String, Value>,
) -> Result<Option<Value>, String> {
    match evaluate(expression, values)? {
        Value::Null => Ok(None),
        other => Ok(Some(other)),
    }
}

fn evaluate_binary(left: Value, operator: BinaryOp, right: Value) -> Result<Value, String> {
    match operator {
        BinaryOp::Or => Ok(Value::Bool(boolean(&left)? || boolean(&right)?)),
        BinaryOp::And => Ok(Value::Bool(boolean(&left)? && boolean(&right)?)),
        BinaryOp::Equal => Ok(Value::Bool(left == right)),
        BinaryOp::NotEqual => Ok(Value::Bool(left != right)),
        BinaryOp::Greater => compare(&left, &right, |value| value.is_gt()),
        BinaryOp::GreaterEqual => compare(&left, &right, |value| value.is_ge()),
        BinaryOp::Less => compare(&left, &right, |value| value.is_lt()),
        BinaryOp::LessEqual => compare(&left, &right, |value| value.is_le()),
        BinaryOp::Add => arithmetic(left, right, |left, right| left + right, i64::checked_add),
        BinaryOp::Subtract => arithmetic(left, right, |left, right| left - right, i64::checked_sub),
        BinaryOp::Multiply => arithmetic(left, right, |left, right| left * right, i64::checked_mul),
        BinaryOp::Divide => {
            if let (Some(left), Some(right)) = (left.as_i64(), right.as_i64()) {
                if right == 0 {
                    return Err("division by zero".into());
                }
                return left
                    .checked_div(right)
                    .map(|value| Value::Number(Number::from(value)))
                    .ok_or_else(|| "integer division overflow".into());
            }
            let divisor = number(&right)?;
            if divisor == 0.0 {
                return Err("division by zero".into());
            }
            number_value(number(&left)? / divisor)
        }
    }
}

fn arithmetic(
    left: Value,
    right: Value,
    float: impl Fn(f64, f64) -> f64,
    integer: impl Fn(i64, i64) -> Option<i64>,
) -> Result<Value, String> {
    if let (Some(left), Some(right)) = (left.as_i64(), right.as_i64()) {
        return integer(left, right)
            .map(|value| Value::Number(Number::from(value)))
            .ok_or_else(|| "integer arithmetic overflow".into());
    }
    number_value(float(number(&left)?, number(&right)?))
}

fn compare(
    left: &Value,
    right: &Value,
    predicate: impl Fn(std::cmp::Ordering) -> bool,
) -> Result<Value, String> {
    let ordering = if left.is_number() && right.is_number() {
        number(left)?
            .partial_cmp(&number(right)?)
            .ok_or_else(|| "numbers cannot be compared".to_string())?
    } else if let (Some(left), Some(right)) = (left.as_str(), right.as_str()) {
        left.cmp(right)
    } else {
        return Err("comparison requires two numbers or two strings".into());
    };
    Ok(Value::Bool(predicate(ordering)))
}

fn boolean(value: &Value) -> Result<bool, String> {
    value
        .as_bool()
        .ok_or_else(|| "logical operator requires bool".into())
}

fn number(value: &Value) -> Result<f64, String> {
    value
        .as_f64()
        .ok_or_else(|| "arithmetic operator requires number".into())
}

fn number_value(value: f64) -> Result<Value, String> {
    Number::from_f64(value)
        .map(Value::Number)
        .ok_or_else(|| "floating-point result is not finite".into())
}

fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let characters = source.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut cursor = 0;
    while cursor < characters.len() {
        let character = characters[cursor];
        if character.is_whitespace() {
            cursor += 1;
            continue;
        }
        if character.is_ascii_alphabetic() || character == '_' {
            let start = cursor;
            cursor += 1;
            while cursor < characters.len()
                && (characters[cursor].is_ascii_alphanumeric()
                    || matches!(characters[cursor], '_' | '.' | '-'))
            {
                cursor += 1;
            }
            let value = characters[start..cursor].iter().collect::<String>();
            tokens.push(match value.as_str() {
                "true" => Token::Bool(true),
                "false" => Token::Bool(false),
                "if" => Token::If,
                "then" => Token::Then,
                "else" => Token::Else,
                _ => Token::Ident(value),
            });
            continue;
        }
        if character.is_ascii_digit() {
            let start = cursor;
            let mut decimal = false;
            cursor += 1;
            while cursor < characters.len()
                && (characters[cursor].is_ascii_digit() || (!decimal && characters[cursor] == '.'))
            {
                if characters[cursor] == '.' {
                    decimal = true;
                }
                cursor += 1;
            }
            let value = characters[start..cursor].iter().collect::<String>();
            tokens.push(if decimal {
                Token::Float(
                    value
                        .parse()
                        .map_err(|_| format!("invalid float literal '{value}'"))?,
                )
            } else {
                Token::Int(
                    value
                        .parse()
                        .map_err(|_| format!("invalid integer literal '{value}'"))?,
                )
            });
            continue;
        }
        if character == '"' {
            let start = cursor;
            cursor += 1;
            let mut escaped = false;
            while cursor < characters.len() {
                let current = characters[cursor];
                cursor += 1;
                if escaped {
                    escaped = false;
                } else if current == '\\' {
                    escaped = true;
                } else if current == '"' {
                    break;
                }
            }
            let raw = characters[start..cursor].iter().collect::<String>();
            let value = serde_json::from_str::<String>(&raw)
                .map_err(|_| "invalid JSON string literal".to_string())?;
            tokens.push(Token::String(value));
            continue;
        }
        let next = characters.get(cursor + 1).copied();
        let (token, width) = match (character, next) {
            ('|', Some('|')) => (Token::Or, 2),
            ('&', Some('&')) => (Token::And, 2),
            ('=', Some('=')) => (Token::Equal, 2),
            ('!', Some('=')) => (Token::NotEqual, 2),
            ('>', Some('=')) => (Token::GreaterEqual, 2),
            ('<', Some('=')) => (Token::LessEqual, 2),
            ('>', _) => (Token::Greater, 1),
            ('<', _) => (Token::Less, 1),
            ('+', _) => (Token::Plus, 1),
            ('-', _) => (Token::Minus, 1),
            ('*', _) => (Token::Star, 1),
            ('/', _) => (Token::Slash, 1),
            ('!', _) => (Token::Bang, 1),
            ('(', _) => (Token::LeftParen, 1),
            (')', _) => (Token::RightParen, 1),
            ('[', _) => (Token::LeftBracket, 1),
            (']', _) => (Token::RightBracket, 1),
            (',', _) => (Token::Comma, 1),
            _ => return Err(format!("unexpected character '{character}' in expression")),
        };
        tokens.push(token);
        cursor += width;
    }
    tokens.push(Token::End);
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.cursor]
    }

    fn take(&mut self) -> Token {
        let token = self.tokens[self.cursor].clone();
        self.cursor += 1;
        token
    }

    fn parse_conditional(&mut self) -> Result<Expr, String> {
        if self.peek() != &Token::If {
            return self.parse_or();
        }
        self.take();
        let condition = self.parse_or()?;
        if self.take() != Token::Then {
            return Err("conditional expression requires 'then'".into());
        }
        let when_true = self.parse_conditional()?;
        if self.take() != Token::Else {
            return Err("conditional expression requires 'else'".into());
        }
        let when_false = self.parse_conditional()?;
        Ok(Expr::Conditional {
            condition: Box::new(condition),
            when_true: Box::new(when_true),
            when_false: Box::new(when_false),
        })
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        self.binary(Self::parse_and, &[Token::Or], &[BinaryOp::Or])
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        self.binary(Self::parse_equality, &[Token::And], &[BinaryOp::And])
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        self.binary(
            Self::parse_comparison,
            &[Token::Equal, Token::NotEqual],
            &[BinaryOp::Equal, BinaryOp::NotEqual],
        )
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        self.binary(
            Self::parse_term,
            &[
                Token::Greater,
                Token::GreaterEqual,
                Token::Less,
                Token::LessEqual,
            ],
            &[
                BinaryOp::Greater,
                BinaryOp::GreaterEqual,
                BinaryOp::Less,
                BinaryOp::LessEqual,
            ],
        )
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        self.binary(
            Self::parse_factor,
            &[Token::Plus, Token::Minus],
            &[BinaryOp::Add, BinaryOp::Subtract],
        )
    }

    fn parse_factor(&mut self) -> Result<Expr, String> {
        self.binary(
            Self::parse_unary,
            &[Token::Star, Token::Slash],
            &[BinaryOp::Multiply, BinaryOp::Divide],
        )
    }

    fn binary(
        &mut self,
        operand: fn(&mut Self) -> Result<Expr, String>,
        tokens: &[Token],
        operators: &[BinaryOp],
    ) -> Result<Expr, String> {
        let mut expression = operand(self)?;
        while let Some(index) = tokens.iter().position(|token| token == self.peek()) {
            self.take();
            let right = operand(self)?;
            expression = Expr::Binary(Box::new(expression), operators[index], Box::new(right));
        }
        Ok(expression)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Token::Bang => {
                self.take();
                Ok(Expr::Unary(UnaryOp::Not, Box::new(self.parse_unary()?)))
            }
            Token::Minus => {
                self.take();
                Ok(Expr::Unary(UnaryOp::Negate, Box::new(self.parse_unary()?)))
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.take() {
            Token::Ident(value) => Ok(Expr::Path(value.split('.').map(str::to_string).collect())),
            Token::Int(value) => Ok(Expr::Int(value)),
            Token::Float(value) => Ok(Expr::Float(value)),
            Token::String(value) => Ok(Expr::String(value)),
            Token::Bool(value) => Ok(Expr::Bool(value)),
            Token::LeftBracket => {
                let mut items = Vec::new();
                if self.peek() == &Token::RightBracket {
                    self.take();
                    return Ok(Expr::List(items));
                }
                loop {
                    items.push(self.parse_conditional()?);
                    match self.take() {
                        Token::Comma => {}
                        Token::RightBracket => break,
                        _ => return Err("list literal requires ',' or ']'".into()),
                    }
                }
                Ok(Expr::List(items))
            }
            Token::LeftParen => {
                let expression = self.parse_conditional()?;
                if self.take() != Token::RightParen {
                    return Err("missing closing ')'".into());
                }
                Ok(expression)
            }
            _ => Err("expected expression value".into()),
        }
    }
}

fn is_uuid_v4_builtin(path: &[String]) -> bool {
    path.len() == 2 && path[0] == "uuid" && path[1] == "v4"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_arithmetic_with_precedence() {
        let expression = parse("input.income - input.expense * 2").unwrap();
        let values = BTreeMap::from([(
            "input".into(),
            serde_json::json!({"income": 100, "expense": 30}),
        )]);
        assert_eq!(evaluate(&expression, &values).unwrap(), 40);
    }

    #[test]
    fn evaluates_boolean_requirements() {
        let expression = parse("input.amount > 0 && input.enabled == true").unwrap();
        let values = BTreeMap::from([(
            "input".into(),
            serde_json::json!({"amount": 25, "enabled": true}),
        )]);
        assert_eq!(evaluate(&expression, &values).unwrap(), true);
    }

    #[test]
    fn integer_division_preserves_the_inferred_type() {
        let values = BTreeMap::new();
        assert_eq!(evaluate(&parse("5 / 2").unwrap(), &values).unwrap(), 2);
        assert_eq!(evaluate(&parse("5.0 / 2").unwrap(), &values).unwrap(), 2.5);
    }

    #[test]
    fn conditional_evaluates_only_the_selected_value() {
        let expression =
            parse("if input.income > 0 then input.income else -input.expense").unwrap();
        let values = BTreeMap::from([(
            "input".into(),
            serde_json::json!({"income": 120, "expense": 45}),
        )]);
        assert_eq!(evaluate(&expression, &values).unwrap(), 120);
    }

    #[test]
    fn evaluates_uuid_v4_builtin() {
        let expression = parse("uuid.v4").unwrap();
        let first = evaluate(&expression, &BTreeMap::new()).unwrap();
        let second = evaluate(&expression, &BTreeMap::new()).unwrap();
        assert_ne!(first, second);
        assert!(first.as_str().unwrap().contains('-'));
    }

    #[test]
    fn evaluate_optional_absent_path_is_none() {
        let values = BTreeMap::from([("input".into(), serde_json::json!({"nome": "x"}))]);
        assert_eq!(
            evaluate_optional(&parse("input.priorita").unwrap(), &values).unwrap(),
            None
        );
    }

    #[test]
    fn evaluate_reads_absent_object_member_as_null() {
        let values = BTreeMap::from([("input".into(), serde_json::json!({"nome": "x"}))]);
        assert_eq!(
            evaluate(&parse("input.priorita").unwrap(), &values).unwrap(),
            Value::Null
        );
    }

    #[test]
    fn evaluate_absent_optional_compares_as_not_equal() {
        let values = BTreeMap::from([("input".into(), serde_json::json!({"nome": "x"}))]);
        assert_eq!(
            evaluate(&parse("input.stato != \"attivo\"").unwrap(), &values).unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn evaluate_unknown_root_still_errors() {
        let values = BTreeMap::new();
        assert!(evaluate(&parse("input.priorita").unwrap(), &values).is_err());
    }

    #[test]
    fn evaluate_optional_present_path_is_some() {
        let values = BTreeMap::from([("input".into(), serde_json::json!({"priorita": 5}))]);
        assert_eq!(
            evaluate_optional(&parse("input.priorita").unwrap(), &values).unwrap(),
            Some(serde_json::json!(5))
        );
    }

    #[test]
    fn evaluate_optional_null_path_is_none() {
        let values = BTreeMap::from([("input".into(), serde_json::json!({"priorita": null}))]);
        assert_eq!(
            evaluate_optional(&parse("input.priorita").unwrap(), &values).unwrap(),
            None
        );
    }

    #[test]
    fn evaluate_optional_propagates_real_errors() {
        let values = BTreeMap::from([("input".into(), serde_json::json!({"a": "x"}))]);
        assert!(evaluate_optional(&parse("input.a + 1").unwrap(), &values).is_err());
    }

    #[test]
    fn evaluates_nested_list_literals() {
        let expression = parse("[\"sales\", if true then \"support\" else \"other\"]").unwrap();
        assert_eq!(
            evaluate(&expression, &BTreeMap::new()).unwrap(),
            serde_json::json!(["sales", "support"])
        );
    }
}
