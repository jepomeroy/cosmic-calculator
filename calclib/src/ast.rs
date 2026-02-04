use crate::token::Token;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Expression {
    Integer {
        value: i64,
    },
    Infix {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    Prefix {
        operator: Token,
        right: Box<Expression>,
    },
    Grouped {
        expression: Box<Expression>,
    },
}

fn prefix_expression() -> Expression {
    todo!()
}
