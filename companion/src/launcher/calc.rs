/// A small calculator: + - * / % ^, parentheses, implicit products, and
/// a few functions and constants. `None` when the text is not math.
pub fn evaluate(text: &str) -> Option<f64> {
    let text = text.trim().trim_start_matches('=').replace(',', ".");
    let tokens = tokenize(&text)?;
    let is_math = tokens.iter().any(|t| match t {
        Token::Op(_) | Token::Open => true,
        Token::Name(n) => FUNCTIONS.contains(&n.as_str()),
        _ => false,
    });
    if !is_math {
        return None;
    }
    let mut parser = Parser { tokens, at: 0 };
    let value = parser.sum()?;
    (parser.at == parser.tokens.len() && value.is_finite()).then_some(value)
}

/// The result as people write it: no trailing zeros, at most 10 decimals.
pub fn format(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        return format!("{}", value as i64);
    }
    let text = format!("{value:.10}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

const FUNCTIONS: [&str; 13] = [
    "sqrt", "raiz", "sin", "sen", "cos", "tan", "tg", "ln", "log", "abs", "round", "floor", "ceil",
];

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Num(f64),
    Op(char),
    Open,
    Close,
    Name(String),
}

fn tokenize(text: &str) -> Option<Vec<Token>> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' => i += 1,
            '0'..='9' | '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let number: String = chars[start..i].iter().collect();
                tokens.push(Token::Num(number.parse().ok()?));
            }
            '+' | '-' | '*' | '/' | '%' | '^' | 'x' | '×' | '÷' => {
                let op = match c {
                    'x' | '×' => '*',
                    '÷' => '/',
                    other => other,
                };
                if c == 'x' && chars.get(i + 1).is_some_and(|n| n.is_alphabetic()) {
                    return None;
                }
                tokens.push(Token::Op(op));
                i += 1;
            }
            '(' => {
                tokens.push(Token::Open);
                i += 1;
            }
            ')' => {
                tokens.push(Token::Close);
                i += 1;
            }
            c if c.is_alphabetic() || c == 'π' => {
                let start = i;
                while i < chars.len() && (chars[i].is_alphabetic() || chars[i] == 'π') {
                    i += 1;
                }
                tokens.push(Token::Name(
                    chars[start..i].iter().collect::<String>().to_lowercase(),
                ));
            }
            _ => return None,
        }
    }
    Some(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }

    fn sum(&mut self) -> Option<f64> {
        let mut value = self.product()?;
        while let Some(Token::Op(op @ ('+' | '-'))) = self.peek().cloned() {
            self.at += 1;
            let rhs = self.product()?;
            value = if op == '+' { value + rhs } else { value - rhs };
        }
        Some(value)
    }

    fn product(&mut self) -> Option<f64> {
        let mut value = self.unary()?;
        loop {
            match self.peek().cloned() {
                Some(Token::Op(op @ ('*' | '/' | '%'))) => {
                    self.at += 1;
                    let rhs = self.unary()?;
                    value = match op {
                        '*' => value * rhs,
                        '/' => value / rhs,
                        _ => value % rhs,
                    };
                }
                Some(Token::Open | Token::Name(_) | Token::Num(_)) => value *= self.unary()?,
                _ => return Some(value),
            }
        }
    }

    fn unary(&mut self) -> Option<f64> {
        match self.peek() {
            Some(Token::Op('-')) => {
                self.at += 1;
                Some(-self.unary()?)
            }
            Some(Token::Op('+')) => {
                self.at += 1;
                self.unary()
            }
            _ => self.power(),
        }
    }

    fn power(&mut self) -> Option<f64> {
        let base = self.atom()?;
        if let Some(Token::Op('^')) = self.peek() {
            self.at += 1;
            let exponent = self.unary()?;
            return Some(base.powf(exponent));
        }
        Some(base)
    }

    fn atom(&mut self) -> Option<f64> {
        let token = self.peek()?.clone();
        self.at += 1;
        match token {
            Token::Num(n) => Some(n),
            Token::Open => {
                let value = self.sum()?;
                if self.peek() == Some(&Token::Close) {
                    self.at += 1;
                }
                Some(value)
            }
            Token::Name(name) => match name.as_str() {
                "pi" | "π" => Some(std::f64::consts::PI),
                "e" => Some(std::f64::consts::E),
                "tau" => Some(std::f64::consts::TAU),
                _ => {
                    let f: fn(f64) -> f64 = match name.as_str() {
                        "sqrt" | "raiz" => f64::sqrt,
                        "sin" | "sen" => f64::sin,
                        "cos" => f64::cos,
                        "tan" | "tg" => f64::tan,
                        "ln" => f64::ln,
                        "log" => f64::log10,
                        "abs" => f64::abs,
                        "round" => f64::round,
                        "floor" => f64::floor,
                        "ceil" => f64::ceil,
                        _ => return None,
                    };
                    Some(f(self.power()?))
                }
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn math() {
        assert_eq!(evaluate("2+2*3"), Some(8.0));
        assert_eq!(evaluate("(1+2)^2"), Some(9.0));
        assert_eq!(evaluate("2(3)"), Some(6.0));
        assert_eq!(evaluate("-2^2"), Some(-4.0));
        assert_eq!(evaluate("10 % 3"), Some(1.0));
        assert_eq!(evaluate("sqrt 16 + 1"), Some(5.0));
        assert_eq!(evaluate("1,5 x 2"), Some(3.0));
        assert_eq!(evaluate("firefox"), None);
        assert_eq!(evaluate("42"), None);
        assert_eq!(evaluate("=42"), None);
        assert_eq!(evaluate("e"), None);
        assert_eq!(evaluate("pi*2").map(format), Some("6.2831853072".into()));
        assert_eq!(format(0.1 + 0.2), "0.3");
        assert_eq!(format(2.0 * std::f64::consts::PI), "6.2831853072");
    }
}
