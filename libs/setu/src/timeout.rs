use std::{fmt, str::FromStr, time::Duration};

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Timeout {
    Hour(u32),
    Minute(u32),
    Second(u32),
    Millisecond(u32),
}

impl fmt::Display for Timeout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Timeout::Hour(hours) => write!(f, "{hours}H"),
            Timeout::Minute(mins) => write!(f, "{mins}M"),
            Timeout::Second(secs) => write!(f, "{secs}S"),
            Timeout::Millisecond(ms) => write!(f, "{ms}m"),
        }
    }
}

impl Timeout {
    pub fn duration(self) -> Duration {
        match self {
            Timeout::Hour(hours) => Duration::from_hours(hours as u64),
            Timeout::Minute(mins) => Duration::from_mins(mins as u64),
            Timeout::Second(secs) => Duration::from_secs(secs as u64),
            Timeout::Millisecond(ms) => Duration::from_millis(ms as u64),
        }
    }
}

impl FromStr for Timeout {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.len() < 2 {
            return Err("timeout: invalid format");
        }

        let (num_part, unit) = input.split_at(input.len() - 1);
        let value: u32 = num_part.parse().map_err(|_| "timeout: invalid number")?;

        match unit {
            "H" => Ok(Timeout::Hour(value)),
            "M" => Ok(Timeout::Minute(value)),
            "S" => Ok(Timeout::Second(value)),
            "m" => Ok(Timeout::Millisecond(value)),
            _ => Err("timeout: unknown unit"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(unit: &str, timeout: Timeout) {
        let s = timeout.to_string();
        assert_eq!(Timeout::from_str(&s).unwrap(), timeout);
        assert_eq!(s, unit);
    }

    #[test]
    fn valid_cases() {
        check("1H", Timeout::Hour(1));
        check("2M", Timeout::Minute(2));
        check("3S", Timeout::Second(3));
        check("10m", Timeout::Millisecond(10));
    }

    #[test]
    fn invalid_cases() {
        assert!(Timeout::from_str(" 1H ").is_err());
        assert!(Timeout::from_str("1").is_err());
        assert!(Timeout::from_str("S").is_err());
        assert!(Timeout::from_str("5X").is_err());
        assert!(Timeout::from_str("aS").is_err());
    }
}
