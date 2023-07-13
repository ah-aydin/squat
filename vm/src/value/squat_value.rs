use std::fmt;
use crate::object::SquatObject;
use super::squat_type::SquatType;

#[derive(Debug, Clone, PartialEq)]
pub enum SquatValue {
    Nil,
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Object(SquatObject),
    Type(SquatType)
}

impl SquatValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            SquatValue::Bool(true) => true,
            SquatValue::Bool(false) | SquatValue::Nil => false,
            _ => true
        }
    }

    pub fn get_type(&self) -> SquatType {
        match self {
            SquatValue::Nil => SquatType::Nil,
            SquatValue::Int(_) => SquatType::Int,
            SquatValue::Float(_) => SquatType::Float,
            SquatValue::String(_) => SquatType::String,
            SquatValue::Bool(_) => SquatType::Bool,
            SquatValue::Object(obj) => obj.get_type(),
            SquatValue::Type(_) => SquatType::Type,
        }
    }
}

impl std::ops::Add<SquatValue> for SquatValue {
    type Output = SquatValue;

    fn add(self, rhs: SquatValue) -> Self::Output {
        match (self, rhs) {
            (SquatValue::Int(i1), SquatValue::Int(i2))
                => SquatValue::Int(i1 + i2),
            (SquatValue::Float(f1), SquatValue::Float(f2))
                => SquatValue::Float(f1 + f2),
            (SquatValue::Int(i), SquatValue::Float(f))
                => SquatValue::Float((i as f64) + f),
            (SquatValue::Float(f), SquatValue::Int(i))
                => SquatValue::Float(f + (i as f64)),
            (SquatValue::String(s1), SquatValue::String(s2))
                => SquatValue::String(s1 + &s2),
            (SquatValue::String(s), value)
                => SquatValue::String(s + &value.to_string()),
            (value, SquatValue::String(s))
                => SquatValue::String(value.to_string() + &s),
            _ => unreachable!()
        }
    }
}

impl std::ops::Sub<SquatValue> for SquatValue {
    type Output = SquatValue;

    fn sub(self, rhs: SquatValue) -> Self::Output {
        match (self, rhs) {
            (SquatValue::Int(i1), SquatValue::Int(i2))
                => SquatValue::Int(i1 - i2),
            (SquatValue::Float(f1), SquatValue::Float(f2))
                => SquatValue::Float(f1 - f2),
            (SquatValue::Int(i), SquatValue::Float(f))
                => SquatValue::Float((i as f64) - f),
            (SquatValue::Float(f), SquatValue::Int(i))
                => SquatValue::Float(f - (i as f64)),
            _ => unreachable!()
        }
    }
}

impl std::ops::Mul<SquatValue> for SquatValue {
    type Output = SquatValue;

    fn mul(self, rhs: SquatValue) -> Self::Output {
        match (self, rhs) {
            (SquatValue::Int(i1), SquatValue::Int(i2))
                => SquatValue::Int(i1 * i2),
            (SquatValue::Float(f1), SquatValue::Float(f2))
                => SquatValue::Float(f1 * f2),
            (SquatValue::Int(i), SquatValue::Float(f))
                => SquatValue::Float((i as f64) * f),
            (SquatValue::Float(f), SquatValue::Int(i))
                => SquatValue::Float(f * (i as f64)),
            _ => unreachable!()
        }
    }
}

impl std::ops::Div<SquatValue> for SquatValue {
    type Output = SquatValue;

    fn div(self, rhs: SquatValue) -> Self::Output {
        match (self, rhs) {
            (SquatValue::Int(i1), SquatValue::Int(i2))
                => SquatValue::Int(i1 / i2),
            (SquatValue::Float(f1), SquatValue::Float(f2))
                => SquatValue::Float(f1 / f2),
            (SquatValue::Int(i), SquatValue::Float(f))
                => SquatValue::Float((i as f64) / f),
            (SquatValue::Float(f), SquatValue::Int(i))
                => SquatValue::Float(f / (i as f64)),
            _ => unreachable!()
        }
    }
}

impl PartialOrd for SquatValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (SquatValue::Nil, SquatValue::Nil) => Some(std::cmp::Ordering::Equal),
            (SquatValue::Int(i1), SquatValue::Int(i2)) => i1.partial_cmp(i2),
            (SquatValue::Float(f1), SquatValue::Float(f2)) => f1.partial_cmp(f2),
            (SquatValue::String(s1), SquatValue::String(s2)) => s1.partial_cmp(s2),
            _ => None
        }
    }
}

impl fmt::Display for SquatValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SquatValue::Nil             => write!(f, "Nil"),
            SquatValue::Int(value)      => write!(f, "{}", value),
            SquatValue::Float(value)    => write!(f, "{}", value),
            SquatValue::Bool(value)     => write!(f, "{}", value),
            SquatValue::String(value)   => write!(f, "{}", value),
            SquatValue::Object(object)  => write!(f, "{}", object.to_string()),
            SquatValue::Type(t)         => write!(f, "{}", t.to_string()),
        }
    }
}

impl Default for SquatValue {
    fn default() -> Self {
        SquatValue::Nil
    }
}

#[cfg(test)]
mod test
{
    use super::*;

    #[test]
    fn int_int() {
        let v1 = SquatValue::Int(10);
        let v2 = SquatValue::Int(2);

        let sum = v1.clone() + v2.clone();
        let sub = v1.clone() - v2.clone();
        let mul = v1.clone() * v2.clone();
        let div = v1.clone() / v2.clone();
        assert_eq!(sum, SquatValue::Int(12));
        assert_eq!(sub, SquatValue::Int(8));
        assert_eq!(mul, SquatValue::Int(20));
        assert_eq!(div, SquatValue::Int(5));
    }

    #[test]
    fn float_float() {
        let v1 = SquatValue::Float(10.);
        let v2 = SquatValue::Float(2.5);

        let sum = v1.clone() + v2.clone();
        let sub = v1.clone() - v2.clone();
        let mul = v1.clone() * v2.clone();
        let div = v1.clone() / v2.clone();
        assert_eq!(sum, SquatValue::Float(12.5));
        assert_eq!(sub, SquatValue::Float(7.5));
        assert_eq!(mul, SquatValue::Float(25.));
        assert_eq!(div, SquatValue::Float(4.));
    }

    #[test]
    fn int_float() {
        let v1 = SquatValue::Int(10);
        let v2 = SquatValue::Float(2.5);

        let sum = v1.clone() + v2.clone();
        let sub = v1.clone() - v2.clone();
        let mul = v1.clone() * v2.clone();
        let div = v1.clone() / v2.clone();
        assert_eq!(sum, SquatValue::Float(12.5));
        assert_eq!(sub, SquatValue::Float(7.5));
        assert_eq!(mul, SquatValue::Float(25.));
        assert_eq!(div, SquatValue::Float(4.));
    }

    #[test]
    fn float_int() {
        let v1 = SquatValue::Float(2.5);
        let v2 = SquatValue::Int(10);

        let sum = v1.clone() + v2.clone();
        let sub = v1.clone() - v2.clone();
        let mul = v1.clone() * v2.clone();
        let div = v1.clone() / v2.clone();
        assert_eq!(sum, SquatValue::Float(12.5));
        assert_eq!(sub, SquatValue::Float(-7.5));
        assert_eq!(mul, SquatValue::Float(25.));
        assert_eq!(div, SquatValue::Float(0.25));
    }

    #[test]
    fn string_anything() {
        let v1 = SquatValue::String("string".to_string());

        let v2 = SquatValue::Int(10);
        assert_eq!(v1.clone() + v2.clone(), SquatValue::String("string10".to_string()));

        let v2 = SquatValue::Float(10.2);
        assert_eq!(v1.clone() + v2.clone(), SquatValue::String("string10.2".to_string()));

        let v2 = SquatValue::Bool(false);
        assert_eq!(v1.clone() + v2.clone(), SquatValue::String("stringfalse".to_string()));
    }

    #[test]
    fn anything_string() {
        let v1 = SquatValue::String("string".to_string());

        let v2 = SquatValue::Int(10);
        assert_eq!(v2.clone() + v1.clone(), SquatValue::String("10string".to_string()));

        let v2 = SquatValue::Float(10.2);
        assert_eq!(v2.clone() + v1.clone(), SquatValue::String("10.2string".to_string()));

        let v2 = SquatValue::Bool(false);
        assert_eq!(v2.clone() + v1.clone(), SquatValue::String("falsestring".to_string()));
    }
}
