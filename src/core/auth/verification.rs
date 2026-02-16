use std::cmp::max;

use super::error::Error;

///
/// Represents the result of the auth verification process. It can either
/// succeed or fail with an Error. On success, it may carry JWT claims.
#[derive(Clone, PartialEq, Debug)]
pub enum Verification {
    Succeed(Option<serde_json::Value>),
    Fail(Error),
}

impl Verification {
    pub fn fail(error: Error) -> Self {
        Verification::Fail(error)
    }

    pub fn succeed() -> Self {
        Verification::Succeed(None)
    }

    pub fn succeed_with_claims(claims: serde_json::Value) -> Self {
        Verification::Succeed(Some(claims))
    }

    #[allow(dead_code)]
    pub fn claims(&self) -> Option<&serde_json::Value> {
        match self {
            Verification::Succeed(claims) => claims.as_ref(),
            Verification::Fail(_) => None,
        }
    }

    #[allow(dead_code)]
    pub fn fold(self, on_success: Self, on_error: impl Fn(Error) -> Self) -> Self {
        match self {
            Verification::Succeed(_) => on_success,
            Verification::Fail(err) => on_error(err),
        }
    }

    pub fn or(&self, other: Self) -> Self {
        match self {
            Verification::Succeed(claims) => Verification::Succeed(claims.clone()),
            Verification::Fail(this) => match other {
                Verification::Succeed(claims) => Verification::Succeed(claims),
                Verification::Fail(that) => Verification::Fail(max(this.clone(), that)),
            },
        }
    }

    pub fn and(self, other: Self) -> Self {
        match self {
            Verification::Succeed(left_claims) => match other {
                Verification::Succeed(right_claims) => {
                    let merged = merge_claims(left_claims, right_claims);
                    Verification::Succeed(merged)
                }
                fail => fail,
            },
            Verification::Fail(_) => self,
        }
    }

    pub fn from_result<A, E>(
        result: Result<A, E>,
        on_success: impl FnOnce(A) -> Verification,
        on_err: impl FnOnce(E) -> Verification,
    ) -> Self {
        match result {
            Ok(data) => on_success(data),
            Err(err) => on_err(err),
        }
    }

    #[allow(dead_code)]
    pub fn to_result(&self) -> Result<(), Error> {
        match self {
            Verification::Succeed(_) => Ok(()),
            Verification::Fail(err) => Err(err.clone()),
        }
    }

    pub fn to_result_with_claims(&self) -> Result<Option<serde_json::Value>, Error> {
        match self {
            Verification::Succeed(claims) => Ok(claims.clone()),
            Verification::Fail(err) => Err(err.clone()),
        }
    }
}

fn merge_claims(
    left: Option<serde_json::Value>,
    right: Option<serde_json::Value>,
) -> Option<serde_json::Value> {
    match (left, right) {
        (Some(serde_json::Value::Object(mut l)), Some(serde_json::Value::Object(r))) => {
            for (k, v) in r {
                l.insert(k, v);
            }
            Some(serde_json::Value::Object(l))
        }
        (Some(l), None) => Some(l),
        (None, Some(r)) => Some(r),
        (None, None) => None,
        // If either side is not an object, prefer the left side
        (Some(l), Some(r)) => {
            tracing::warn!(
                "merge_claims: non-object claims detected, dropping right side. left={}, right={}",
                l,
                r
            );
            Some(l)
        }
    }
}
