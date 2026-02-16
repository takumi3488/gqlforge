use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while1};
use nom::character::complete::{char, multispace0};
use nom::combinator::{map, value};
use nom::multi::separated_list1;
use nom::sequence::{delimited, preceded, tuple};

use super::{EvalContext, ResolverContextLike};
use crate::core::path::PathString;

#[derive(Clone, Debug, PartialEq)]
pub enum AccessExpr {
    And(Box<AccessExpr>, Box<AccessExpr>),
    Or(Box<AccessExpr>, Box<AccessExpr>),
    Not(Box<AccessExpr>),
    Eq(Operand, Operand),
    Neq(Operand, Operand),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operand {
    Path(Vec<String>),
    StringLiteral(String),
    NumberLiteral(i64),
    BoolLiteral(bool),
}

impl AccessExpr {
    pub fn parse(input: &str) -> Result<Self, String> {
        let (remaining, expr) = parse_or_expr(input.trim())
            .map_err(|e| format!("Failed to parse access expression: {}", e))?;
        let remaining = remaining.trim();
        if !remaining.is_empty() {
            return Err(format!("Unexpected trailing input: '{}'", remaining));
        }
        Ok(expr)
    }

    pub fn evaluate<Ctx: ResolverContextLike + Sync>(
        &self,
        ctx: &EvalContext<'_, Ctx>,
    ) -> Result<bool, String> {
        match self {
            AccessExpr::And(left, right) => Ok(left.evaluate(ctx)? && right.evaluate(ctx)?),
            AccessExpr::Or(left, right) => Ok(left.evaluate(ctx)? || right.evaluate(ctx)?),
            AccessExpr::Not(expr) => Ok(!expr.evaluate(ctx)?),
            AccessExpr::Eq(left, right) => {
                let l = resolve_operand(left, ctx);
                let r = resolve_operand(right, ctx);
                Ok(l == r)
            }
            AccessExpr::Neq(left, right) => {
                let l = resolve_operand(left, ctx);
                let r = resolve_operand(right, ctx);
                Ok(l != r)
            }
        }
    }
}

fn resolve_operand<Ctx: ResolverContextLike + Sync>(
    operand: &Operand,
    ctx: &EvalContext<'_, Ctx>,
) -> Option<String> {
    match operand {
        Operand::Path(segments) => ctx.path_string(segments).map(|s| s.into_owned()),
        Operand::StringLiteral(s) => Some(s.clone()),
        Operand::NumberLiteral(n) => Some(n.to_string()),
        Operand::BoolLiteral(b) => Some(b.to_string()),
    }
}

// Parser functions using nom

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-'
}

fn parse_path(input: &str) -> IResult<&str, Operand> {
    map(
        separated_list1(char('.'), take_while1(is_ident_char)),
        |parts: Vec<&str>| Operand::Path(parts.into_iter().map(String::from).collect()),
    )(input)
}

fn parse_string_literal(input: &str) -> IResult<&str, Operand> {
    let (input, _) = char('\'')(input)?;
    let mut end = 0;
    let bytes = input.as_bytes();
    while end < bytes.len() && bytes[end] != b'\'' {
        end += 1;
    }
    if end >= bytes.len() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char,
        )));
    }
    let s = &input[..end];
    let remaining = &input[end + 1..];
    Ok((remaining, Operand::StringLiteral(s.to_owned())))
}

fn parse_number_literal(input: &str) -> IResult<&str, Operand> {
    let (input, neg) = nom::combinator::opt(char('-'))(input)?;
    let (input, digits) = take_while1(|c: char| c.is_ascii_digit())(input)?;
    let n: i64 = digits.parse().map_err(|_| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
    })?;
    let n = if neg.is_some() { -n } else { n };
    Ok((input, Operand::NumberLiteral(n)))
}

fn parse_bool_literal(input: &str) -> IResult<&str, Operand> {
    alt((
        value(Operand::BoolLiteral(true), tag("true")),
        value(Operand::BoolLiteral(false), tag("false")),
    ))(input)
}

fn parse_operand(input: &str) -> IResult<&str, Operand> {
    let (input, _) = multispace0(input)?;

    alt((
        parse_string_literal,
        parse_bool_literal,
        parse_number_literal,
        parse_path,
    ))(input)
}

fn parse_comparison(input: &str) -> IResult<&str, AccessExpr> {
    let (input, left) = parse_operand(input)?;
    let (input, _) = multispace0(input)?;
    let (input, op) = alt((tag("!="), tag("==")))(input)?;
    let (input, _) = multispace0(input)?;
    let (input, right) = parse_operand(input)?;
    let (input, _) = multispace0(input)?;
    match op {
        "==" => Ok((input, AccessExpr::Eq(left, right))),
        "!=" => Ok((input, AccessExpr::Neq(left, right))),
        _ => unreachable!(),
    }
}

fn parse_not(input: &str) -> IResult<&str, AccessExpr> {
    let (input, _) = multispace0(input)?;
    let (input, _) = char('!')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, expr) = parse_atom(input)?;
    Ok((input, AccessExpr::Not(Box::new(expr))))
}

fn parse_parens(input: &str) -> IResult<&str, AccessExpr> {
    delimited(
        preceded(multispace0, char('(')),
        parse_or_expr,
        preceded(multispace0, char(')')),
    )(input)
}

fn parse_atom(input: &str) -> IResult<&str, AccessExpr> {
    let (input, _) = multispace0(input)?;
    alt((parse_parens, parse_not, parse_comparison))(input)
}

fn parse_and_expr(input: &str) -> IResult<&str, AccessExpr> {
    let (input, first) = parse_atom(input)?;
    let (input, rest) = nom::multi::many0(preceded(
        tuple((multispace0, tag("&&"), multispace0)),
        parse_atom,
    ))(input)?;
    Ok((
        input,
        rest.into_iter().fold(first, |acc, expr| {
            AccessExpr::And(Box::new(acc), Box::new(expr))
        }),
    ))
}

fn parse_or_expr(input: &str) -> IResult<&str, AccessExpr> {
    let (input, first) = parse_and_expr(input)?;
    let (input, rest) = nom::multi::many0(preceded(
        tuple((multispace0, tag("||"), multispace0)),
        parse_and_expr,
    ))(input)?;
    Ok((
        input,
        rest.into_iter().fold(first, |acc, expr| {
            AccessExpr::Or(Box::new(acc), Box::new(expr))
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_eq() {
        let expr = AccessExpr::parse("claims.role == 'admin'").unwrap();
        assert_eq!(
            expr,
            AccessExpr::Eq(
                Operand::Path(vec!["claims".into(), "role".into()]),
                Operand::StringLiteral("admin".into()),
            )
        );
    }

    #[test]
    fn test_parse_neq() {
        let expr = AccessExpr::parse("claims.role != 'guest'").unwrap();
        assert_eq!(
            expr,
            AccessExpr::Neq(
                Operand::Path(vec!["claims".into(), "role".into()]),
                Operand::StringLiteral("guest".into()),
            )
        );
    }

    #[test]
    fn test_parse_and() {
        let expr =
            AccessExpr::parse("claims.role == 'admin' && claims.sub == args.userId").unwrap();
        match expr {
            AccessExpr::And(left, right) => {
                assert_eq!(
                    *left,
                    AccessExpr::Eq(
                        Operand::Path(vec!["claims".into(), "role".into()]),
                        Operand::StringLiteral("admin".into()),
                    )
                );
                assert_eq!(
                    *right,
                    AccessExpr::Eq(
                        Operand::Path(vec!["claims".into(), "sub".into()]),
                        Operand::Path(vec!["args".into(), "userId".into()]),
                    )
                );
            }
            _ => panic!("Expected And"),
        }
    }

    #[test]
    fn test_parse_or() {
        let expr =
            AccessExpr::parse("claims.role == 'admin' || claims.role == 'moderator'").unwrap();
        match expr {
            AccessExpr::Or(_, _) => {}
            _ => panic!("Expected Or"),
        }
    }

    #[test]
    fn test_parse_not() {
        let expr = AccessExpr::parse("!(claims.role == 'guest')").unwrap();
        match expr {
            AccessExpr::Not(inner) => {
                assert_eq!(
                    *inner,
                    AccessExpr::Eq(
                        Operand::Path(vec!["claims".into(), "role".into()]),
                        Operand::StringLiteral("guest".into()),
                    )
                );
            }
            _ => panic!("Expected Not"),
        }
    }

    #[test]
    fn test_parse_parens() {
        let expr = AccessExpr::parse(
            "(claims.role == 'admin' || claims.role == 'mod') && claims.active == true",
        )
        .unwrap();
        match expr {
            AccessExpr::And(left, right) => {
                assert!(matches!(*left, AccessExpr::Or(_, _)));
                assert_eq!(
                    *right,
                    AccessExpr::Eq(
                        Operand::Path(vec!["claims".into(), "active".into()]),
                        Operand::BoolLiteral(true),
                    )
                );
            }
            _ => panic!("Expected And"),
        }
    }

    #[test]
    fn test_parse_number() {
        let expr = AccessExpr::parse("claims.level == 42").unwrap();
        assert_eq!(
            expr,
            AccessExpr::Eq(
                Operand::Path(vec!["claims".into(), "level".into()]),
                Operand::NumberLiteral(42),
            )
        );
    }

    #[test]
    fn test_parse_error() {
        assert!(AccessExpr::parse("").is_err());
        assert!(AccessExpr::parse("==").is_err());
        assert!(AccessExpr::parse("claims.role ==").is_err());
    }

    #[test]
    fn test_parse_trailing_input() {
        assert!(AccessExpr::parse("claims.role == 'admin' garbage").is_err());
    }

    #[test]
    fn test_evaluate_eq_literals() {
        use crate::core::http::RequestContext;
        use crate::core::ir::EmptyResolverContext;

        let runtime = crate::core::runtime::test::init(None);
        let req_ctx = RequestContext::new(runtime);
        req_ctx.set_auth_claims(serde_json::json!({
            "sub": "user123",
            "role": "admin",
        }));

        let res_ctx = EmptyResolverContext {};
        let eval_ctx = EvalContext::new(&req_ctx, &res_ctx);

        let expr = AccessExpr::parse("claims.role == 'admin'").unwrap();
        assert!(expr.evaluate(&eval_ctx).unwrap());

        let expr = AccessExpr::parse("claims.role == 'user'").unwrap();
        assert!(!expr.evaluate(&eval_ctx).unwrap());

        let expr = AccessExpr::parse("claims.role != 'user'").unwrap();
        assert!(expr.evaluate(&eval_ctx).unwrap());
    }

    #[test]
    fn test_evaluate_missing_claims() {
        use crate::core::http::RequestContext;
        use crate::core::ir::EmptyResolverContext;

        let runtime = crate::core::runtime::test::init(None);
        let req_ctx = RequestContext::new(runtime);
        // No claims set

        let res_ctx = EmptyResolverContext {};
        let eval_ctx = EvalContext::new(&req_ctx, &res_ctx);

        let expr = AccessExpr::parse("claims.role == 'admin'").unwrap();
        // Missing claims path resolves to None, 'admin' resolves to Some - not equal
        assert!(!expr.evaluate(&eval_ctx).unwrap());
    }

    #[test]
    fn test_evaluate_and_or() {
        use crate::core::http::RequestContext;
        use crate::core::ir::EmptyResolverContext;

        let runtime = crate::core::runtime::test::init(None);
        let req_ctx = RequestContext::new(runtime);
        req_ctx.set_auth_claims(serde_json::json!({
            "role": "admin",
            "active": "true",
        }));

        let res_ctx = EmptyResolverContext {};
        let eval_ctx = EvalContext::new(&req_ctx, &res_ctx);

        let expr = AccessExpr::parse("claims.role == 'admin' && claims.active == true").unwrap();
        assert!(expr.evaluate(&eval_ctx).unwrap());

        let expr = AccessExpr::parse("claims.role == 'user' || claims.role == 'admin'").unwrap();
        assert!(expr.evaluate(&eval_ctx).unwrap());

        let expr = AccessExpr::parse("claims.role == 'user' && claims.active == true").unwrap();
        assert!(!expr.evaluate(&eval_ctx).unwrap());
    }
}
