use bigdecimal::BigDecimal;
use std::str::FromStr;

#[derive(Debug)]
pub struct GRT(BigDecimal);

impl FromStr for GRT {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let wei: BigDecimal = s.parse().unwrap();
        let factor: BigDecimal = "1000000000000000000".parse().unwrap();
        Ok(GRT(wei / factor))
    }
}
