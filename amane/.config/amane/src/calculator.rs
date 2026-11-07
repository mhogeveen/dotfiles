/*
 * the launcher's calculator: =12*7 shows 84
 *
 * reads numbers (with . or , as the decimal point), + - * / % with the usual
 * precedence, ^ for powers (right to left, so 2^3^2 is 512, and -2^2 is -4),
 * negative numbers and parentheses; spaces don't matter
 */

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    // something that isn't a number, an operator or a parenthesis, or left over
    NotASum,

    DivideByZero,
}

impl Error {
    pub fn message(&self) -> &'static str {
        match self {
            Error::NotASum => "Not a sum",
            Error::DivideByZero => "Can't divide by zero",
        }
    }
}

// the answer as it should be shown and copied, like 84 or 0.3333333333
pub fn calculate(sum: &str) -> Result<String, Error> {
    let characters: Vec<char> = sum.chars().filter(|c| !c.is_whitespace()).collect();

    let mut parser = Parser {
        characters,
        position: 0,
    };

    let value = parser.sum()?;

    // something the parser couldn't place, like a stray ) or letter
    if parser.position != parser.characters.len() {
        return Err(Error::NotASum);
    }

    if !value.is_finite() {
        return Err(Error::NotASum);
    }

    Ok(format(value))
}

struct Parser {
    characters: Vec<char>,
    position: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.characters.get(self.position).copied()
    }

    fn take(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.position += 1;

            true
        } else {
            false
        }
    }

    // terms added and subtracted
    fn sum(&mut self) -> Result<f64, Error> {
        let mut value = self.product()?;

        loop {
            if self.take('+') {
                value += self.product()?;
            } else if self.take('-') {
                value -= self.product()?;
            } else {
                return Ok(value);
            }
        }
    }

    // factors multiplied, divided and taken the remainder of
    fn product(&mut self) -> Result<f64, Error> {
        let mut value = self.signed()?;

        loop {
            if self.take('*') {
                value *= self.signed()?;
            } else if self.take('/') {
                let divisor = self.signed()?;

                if divisor == 0.0 {
                    return Err(Error::DivideByZero);
                }

                value /= divisor;
            } else if self.take('%') {
                let divisor = self.signed()?;

                if divisor == 0.0 {
                    return Err(Error::DivideByZero);
                }

                value %= divisor;
            } else {
                return Ok(value);
            }
        }
    }

    // right to left: 2^3^2 is 2^(3^2); the exponent may be negative, 2^-1 is 0.5
    fn power(&mut self) -> Result<f64, Error> {
        let base = self.atom()?;

        if self.take('^') {
            let exponent = self.signed()?;

            return Ok(base.powf(exponent));
        }

        Ok(base)
    }

    // a minus or plus in front: -3, --3, +3; looser than ^, so -2^2 is -(2^2)
    fn signed(&mut self) -> Result<f64, Error> {
        if self.take('-') {
            return Ok(-self.signed()?);
        }

        if self.take('+') {
            return self.signed();
        }

        self.power()
    }

    fn atom(&mut self) -> Result<f64, Error> {
        if self.take('(') {
            let value = self.sum()?;

            if !self.take(')') {
                return Err(Error::NotASum);
            }

            return Ok(value);
        }

        self.number()
    }

    // digits with at most one decimal point, written . or ,
    fn number(&mut self) -> Result<f64, Error> {
        let start = self.position;
        let mut text = String::new();
        let mut decimal = false;

        while let Some(character) = self.peek() {
            match character {
                '0'..='9' => text.push(character),
                '.' | ',' if !decimal => {
                    decimal = true;
                    text.push('.');
                }
                _ => break,
            }

            self.position += 1;
        }

        if self.position == start || text == "." {
            return Err(Error::NotASum);
        }

        text.parse().map_err(|_| Error::NotASum)
    }
}

// whole numbers plainly, others to 10 significant digits, huge and tiny ones in e-notation
fn format(value: f64) -> String {
    // -0 reads oddly
    let value = if value == 0.0 { 0.0 } else { value };

    let size = value.abs();

    if size != 0.0 && !(1e-6..1e15).contains(&size) {
        return trim_exponent(&format!("{value:.9e}"));
    }

    if value.fract() == 0.0 {
        return format!("{value:.0}");
    }

    // ten significant digits: 3.14 keeps 9 decimals, 0.0314 keeps 11
    let magnitude = size.log10().floor() as i32;
    let after = (9 - magnitude).max(0) as usize;

    trim_zeros(&format!("{value:.after$}"))
}

fn trim_zeros(text: &str) -> String {
    if !text.contains('.') {
        return String::from(text);
    }

    String::from(text.trim_end_matches('0').trim_end_matches('.'))
}

// 1.200000000e21 reads as 1.2e21
fn trim_exponent(text: &str) -> String {
    match text.split_once('e') {
        Some((mantissa, exponent)) => format!("{}e{exponent}", trim_zeros(mantissa)),
        None => String::from(text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn answer(sum: &str) -> String {
        calculate(sum).expect("a sum")
    }

    #[test]
    fn precedence_and_parentheses() {
        assert_eq!(answer("12*7"), "84");
        assert_eq!(answer("1+2*3"), "7");
        assert_eq!(answer("(1+2)*3"), "9");
        assert_eq!(answer("10-4-3"), "3");
        assert_eq!(answer("100/10/5"), "2");
        assert_eq!(answer(" 2 * ( 3 + 4 ) "), "14");
    }

    #[test]
    fn powers_remainders_and_signs() {
        assert_eq!(answer("2^10"), "1024");
        assert_eq!(answer("2^3^2"), "512");
        assert_eq!(answer("-2^2"), "-4");
        assert_eq!(answer("(-2)^2"), "4");
        assert_eq!(answer("2^-1"), "0.5");
        assert_eq!(answer("17%5"), "2");
        assert_eq!(answer("-3+5"), "2");
        assert_eq!(answer("4*-2"), "-8");
        assert_eq!(answer("--3"), "3");
    }

    #[test]
    fn decimals_with_point_or_comma() {
        assert_eq!(answer("2,5*2"), "5");
        assert_eq!(answer("0.1+0.2"), "0.3");
        assert_eq!(answer("1/3"), "0.3333333333");
        assert_eq!(answer("22/7"), "3.142857143");
        assert_eq!(answer(".5+.5"), "1");
    }

    #[test]
    fn large_and_small_answers() {
        assert_eq!(answer("1.2*10^21"), "1.2e21");
        assert_eq!(answer("1/10^9"), "1e-9");
        assert_eq!(answer("0*-1"), "0");
    }

    #[test]
    fn mistakes_are_reported() {
        assert_eq!(calculate("1/0"), Err(Error::DivideByZero));
        assert_eq!(calculate("5%0"), Err(Error::DivideByZero));
        assert_eq!(calculate(""), Err(Error::NotASum));
        assert_eq!(calculate("2+"), Err(Error::NotASum));
        assert_eq!(calculate("(2+3"), Err(Error::NotASum));
        assert_eq!(calculate("2+3)"), Err(Error::NotASum));
        assert_eq!(calculate("abc"), Err(Error::NotASum));
        assert_eq!(calculate("1.2.3"), Err(Error::NotASum));
    }
}
