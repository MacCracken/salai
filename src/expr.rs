//! Expression evaluator for inspector property fields.
//!
//! Wraps [`abaco::Evaluator`] to let users type math expressions
//! (e.g. `2*pi`, `sin(45)`, `1+2`) into numeric editor fields.

use abaco::{EvalError, Evaluator, Value};

/// Evaluate a math expression string and return the result as `f64`.
///
/// Supports arithmetic operators, trig functions, constants (`pi`, `e`, `tau`),
/// and scientific notation. Returns the original value unchanged if the input
/// is already a plain number.
pub fn eval_f64(expr: &str) -> Result<f64, ExprError> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return Err(ExprError::Empty);
    }

    let evaluator = Evaluator::new();
    let value = evaluator.eval(trimmed)?;

    value.as_f64().ok_or(ExprError::NotNumeric(value))
}

/// Evaluate an expression, falling back to a default value on error.
pub fn eval_or(expr: &str, default: f64) -> f64 {
    eval_f64(expr).unwrap_or(default)
}

/// Try to evaluate an expression; if it fails, try parsing as a plain number.
pub fn eval_or_parse(expr: &str) -> Result<f64, ExprError> {
    match eval_f64(expr) {
        Ok(v) => Ok(v),
        Err(_) => expr
            .trim()
            .parse::<f64>()
            .map_err(|_| ExprError::ParseFailed(expr.to_string())),
    }
}

/// Expression evaluation error.
#[derive(Debug)]
pub enum ExprError {
    /// Input string was empty or whitespace-only.
    Empty,
    /// Expression evaluated to a non-numeric value (e.g. text).
    NotNumeric(Value),
    /// Could not evaluate or parse the expression.
    ParseFailed(String),
    /// Evaluation error from abaco.
    Eval(EvalError),
}

impl std::fmt::Display for ExprError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprError::Empty => write!(f, "empty expression"),
            ExprError::NotNumeric(v) => write!(f, "non-numeric result: {v}"),
            ExprError::ParseFailed(s) => write!(f, "cannot parse: {s}"),
            ExprError::Eval(e) => write!(f, "eval error: {e}"),
        }
    }
}

impl std::error::Error for ExprError {}

impl From<EvalError> for ExprError {
    fn from(e: EvalError) -> Self {
        ExprError::Eval(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // eval_f64 — basic arithmetic
    // -----------------------------------------------------------------------

    #[test]
    fn eval_addition() {
        let v = eval_f64("1 + 2").unwrap();
        assert!((v - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn eval_multiplication() {
        let v = eval_f64("3 * 4").unwrap();
        assert!((v - 12.0).abs() < f64::EPSILON);
    }

    #[test]
    fn eval_division() {
        let v = eval_f64("10 / 4").unwrap();
        assert!((v - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn eval_power() {
        let v = eval_f64("2 ^ 10").unwrap();
        assert!((v - 1024.0).abs() < f64::EPSILON);
    }

    #[test]
    fn eval_parentheses() {
        let v = eval_f64("(1 + 2) * 3").unwrap();
        assert!((v - 9.0).abs() < f64::EPSILON);
    }

    #[test]
    fn eval_negative_number() {
        let v = eval_f64("-5").unwrap();
        assert!((v - (-5.0)).abs() < f64::EPSILON);
    }

    // -----------------------------------------------------------------------
    // eval_f64 — constants
    // -----------------------------------------------------------------------

    #[test]
    fn eval_pi() {
        let v = eval_f64("pi").unwrap();
        assert!((v - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn eval_e() {
        let v = eval_f64("e").unwrap();
        assert!((v - std::f64::consts::E).abs() < 1e-10);
    }

    #[test]
    fn eval_tau() {
        let v = eval_f64("tau").unwrap();
        assert!((v - std::f64::consts::TAU).abs() < 1e-10);
    }

    #[test]
    fn eval_2_times_pi() {
        let v = eval_f64("2 * pi").unwrap();
        assert!((v - std::f64::consts::TAU).abs() < 1e-10);
    }

    // -----------------------------------------------------------------------
    // eval_f64 — functions
    // -----------------------------------------------------------------------

    #[test]
    fn eval_sqrt() {
        let v = eval_f64("sqrt(9)").unwrap();
        assert!((v - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn eval_sin() {
        let v = eval_f64("sin(0)").unwrap();
        assert!(v.abs() < 1e-10);
    }

    #[test]
    fn eval_cos() {
        let v = eval_f64("cos(0)").unwrap();
        assert!((v - 1.0).abs() < 1e-10);
    }

    #[test]
    fn eval_abs() {
        let v = eval_f64("abs(-42)").unwrap();
        assert!((v - 42.0).abs() < f64::EPSILON);
    }

    // -----------------------------------------------------------------------
    // eval_f64 — scientific notation
    // -----------------------------------------------------------------------

    #[test]
    fn eval_scientific_notation() {
        let v = eval_f64("1.5e3").unwrap();
        assert!((v - 1500.0).abs() < f64::EPSILON);
    }

    // -----------------------------------------------------------------------
    // eval_f64 — plain numbers
    // -----------------------------------------------------------------------

    #[test]
    fn eval_plain_integer() {
        let v = eval_f64("42").unwrap();
        assert!((v - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn eval_plain_float() {
        let v = eval_f64("3.14").unwrap();
        assert!((v - 3.14).abs() < 1e-10);
    }

    // -----------------------------------------------------------------------
    // eval_f64 — error cases
    // -----------------------------------------------------------------------

    #[test]
    fn eval_empty_string() {
        assert!(matches!(eval_f64(""), Err(ExprError::Empty)));
    }

    #[test]
    fn eval_whitespace_only() {
        assert!(matches!(eval_f64("   "), Err(ExprError::Empty)));
    }

    #[test]
    fn eval_invalid_expression() {
        assert!(eval_f64("+++").is_err());
    }

    #[test]
    fn eval_unknown_function() {
        assert!(eval_f64("foobar(1)").is_err());
    }

    // -----------------------------------------------------------------------
    // eval_or
    // -----------------------------------------------------------------------

    #[test]
    fn eval_or_success() {
        assert!((eval_or("1+1", 0.0) - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn eval_or_fallback() {
        assert!((eval_or("bad", 99.0) - 99.0).abs() < f64::EPSILON);
    }

    #[test]
    fn eval_or_empty_fallback() {
        assert!((eval_or("", 5.0) - 5.0).abs() < f64::EPSILON);
    }

    // -----------------------------------------------------------------------
    // eval_or_parse
    // -----------------------------------------------------------------------

    #[test]
    fn eval_or_parse_expression() {
        let v = eval_or_parse("2*pi").unwrap();
        assert!((v - std::f64::consts::TAU).abs() < 1e-10);
    }

    #[test]
    fn eval_or_parse_plain_number() {
        let v = eval_or_parse("42.5").unwrap();
        assert!((v - 42.5).abs() < f64::EPSILON);
    }

    #[test]
    fn eval_or_parse_fails() {
        assert!(eval_or_parse("not_a_number_or_expr!@#").is_err());
    }
}
