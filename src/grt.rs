use bigdecimal::BigDecimal;
use std::fmt;
use std::ops::{Div, Mul, Sub};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GRT(BigDecimal);

impl FromStr for GRT {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let wei: BigDecimal = s.parse().unwrap();
        let factor: BigDecimal = "1000000000000000000".parse().unwrap();
        Ok(GRT(wei / factor))
    }
}

impl fmt::Display for GRT {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let num = format!("{}", self.0);
        // Calling BigDecimal::round panics. So truncate.
        let mut num = num.as_str();
        if let Some(idx) = num.find(".") {
            num = &num[0..(num.len().min(idx + 4))];
        }
        write!(f, "{} GRT", num)
    }
}

impl Mul<BigDecimal> for GRT {
    type Output = GRT;
    fn mul(self, rhs: BigDecimal) -> Self::Output {
        GRT(self.0 * rhs)
    }
}

impl Div<BigDecimal> for GRT {
    type Output = GRT;
    fn div(self, rhs: BigDecimal) -> Self::Output {
        GRT(self.0 / rhs)
    }
}

impl Sub for GRT {
    type Output = GRT;
    fn sub(self, rhs: GRT) -> Self::Output {
        GRT(self.0 - rhs.0)
    }
}
