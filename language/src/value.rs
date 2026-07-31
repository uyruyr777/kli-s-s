use std::fmt;

/// "Число, близкое к бесконечности" — большое число, состоящее из цепочки
/// "лимбов" (внутренних ячеек). Каждый лимб хранит значение в диапазоне
/// [0, LIMB_BASE). Когда при сложении/умножении лимб переполняется —
/// избыток (перенос) добавляется в следующий лимб; если следующего нет,
/// он создаётся. Именно так ncti растёт "сам в себе".
const LIMB_BASE: i64 = 1_000_000_000_000_000_000; // 10^18

#[derive(Debug, Clone, PartialEq)]
pub struct Ncti {
    /// Лимбы от младшего к старшему, каждый в диапазоне [0, LIMB_BASE)
    pub limbs: Vec<i64>,
}

impl Ncti {
    pub fn zero() -> Self {
        Self { limbs: vec![0] }
    }

    pub fn from_i64(n: i64) -> Self {
        let mut n = n.unsigned_abs() as i128;
        if n == 0 {
            return Self::zero();
        }
        let mut limbs = Vec::new();
        while n > 0 {
            limbs.push((n % LIMB_BASE as i128) as i64);
            n /= LIMB_BASE as i128;
        }
        Self { limbs }
    }

    /// Убрать лишние старшие нулевые лимбы (кроме единственного, если число — 0)
    pub fn normalize(mut self) -> Self {
        while self.limbs.len() > 1 && *self.limbs.last().unwrap() == 0 {
            self.limbs.pop();
        }
        self
    }

    /// Если число помещается в один лимб — вернуть его как i64
    pub fn to_i64(&self) -> Option<i64> {
        let n = self.clone().normalize();
        if n.limbs.len() == 1 {
            Some(n.limbs[0])
        } else {
            None
        }
    }

    pub fn to_decimal_string(&self) -> String {
        let n = self.clone().normalize();
        let mut result = String::new();

        for (i, limb) in n.limbs.iter().enumerate().rev() {
            if i == n.limbs.len() - 1 {
                result.push_str(&limb.to_string());
            } else {
                result.push_str(&format!("{:018}", limb));
            }
        }

        result
    }

    /// Сложение с переносом: когда лимб доходит до предела (LIMB_BASE),
    /// избыток добавляется в следующий (создаётся при необходимости).
    pub fn add(&self, other: &Ncti) -> Ncti {
        let mut result = Vec::new();
        let mut carry: i64 = 0;
        let len = self.limbs.len().max(other.limbs.len());

        for i in 0..len {
            let a = *self.limbs.get(i).unwrap_or(&0);
            let b = *other.limbs.get(i).unwrap_or(&0);
            let mut sum = a + b + carry;
            carry = sum / LIMB_BASE;
            sum %= LIMB_BASE;
            result.push(sum);
        }

        if carry > 0 {
            result.push(carry);
        }

        Ncti { limbs: result }.normalize()
    }

    /// Вычитание (с заимствованием). Предполагается self >= other —
    /// отрицательные ncti пока не поддерживаются.
    pub fn sub(&self, other: &Ncti) -> Ncti {
        let mut result = Vec::new();
        let mut borrow: i64 = 0;

        for i in 0..self.limbs.len() {
            let a = self.limbs[i];
            let b = *other.limbs.get(i).unwrap_or(&0);
            let mut diff = a - b - borrow;
            if diff < 0 {
                diff += LIMB_BASE;
                borrow = 1;
            } else {
                borrow = 0;
            }
            result.push(diff);
        }

        Ncti { limbs: result }.normalize()
    }

    pub fn mul(&self, other: &Ncti) -> Ncti {
        let mut result = vec![0i128; self.limbs.len() + other.limbs.len()];

        for (i, &a) in self.limbs.iter().enumerate() {
            let mut carry: i128 = 0;
            for (j, &b) in other.limbs.iter().enumerate() {
                let cur = result[i + j] + (a as i128) * (b as i128) + carry;
                result[i + j] = cur % LIMB_BASE as i128;
                carry = cur / LIMB_BASE as i128;
            }
            let mut k = i + other.limbs.len();
            while carry > 0 {
                let cur = result[k] + carry;
                result[k] = cur % LIMB_BASE as i128;
                carry = cur / LIMB_BASE as i128;
                k += 1;
            }
        }

        let limbs = result.into_iter().map(|v| v as i64).collect();
        Ncti { limbs }.normalize()
    }

    pub fn cmp(&self, other: &Ncti) -> std::cmp::Ordering {
        let a = self.clone().normalize();
        let b = other.clone().normalize();

        if a.limbs.len() != b.limbs.len() {
            return a.limbs.len().cmp(&b.limbs.len());
        }

        for i in (0..a.limbs.len()).rev() {
            if a.limbs[i] != b.limbs[i] {
                return a.limbs[i].cmp(&b.limbs[i]);
            }
        }

        std::cmp::Ordering::Equal
    }
}

/// Одно типизированное поле json-объекта: хранит и объявленный тип, и
/// текущее значение. Тип нужен, чтобы решать, можно ли присвоить полю
/// новое значение обычным `=` (тип должен совпасть) или для этого нужен
/// явный `.#тип = значение` (смена типа).
#[derive(Debug, Clone, PartialEq)]
pub struct JsonField {
    pub declared_type: ValueType,
    pub value: Value,
}

/// `#json obj {int#"n":1, ...}` — набор типизированных полей.
#[derive(Debug, Clone, PartialEq)]
pub struct JsonObject {
    pub fields: std::collections::HashMap<String, JsonField>,
}

/// Типы значений языка
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Array(Vec<Value>),
    /// "Число, близкое к бесконечности" — см. Ncti выше
    Ncti(Ncti),
    Json(JsonObject),
    Null,
}

/// Возможные типы данных
#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Int,
    Float,
    Bool,
    String,
    Array,
    Ncti,
    Json,
    Null,
}

impl Value {
    /// Получить тип значения
    pub fn get_type(&self) -> ValueType {
        match self {
            Value::Int(_) => ValueType::Int,
            Value::Float(_) => ValueType::Float,
            Value::Bool(_) => ValueType::Bool,
            Value::String(_) => ValueType::String,
            Value::Array(_) => ValueType::Array,
            Value::Ncti(_) => ValueType::Ncti,
            Value::Json(_) => ValueType::Json,
            Value::Null => ValueType::Null,
        }
    }

    /// Преобразовать в bool
    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(v) => *v,
            Value::Int(v) => *v != 0,
            Value::Float(v) => *v != 0.0,
            Value::String(v) => !v.is_empty(),
            Value::Array(v) => !v.is_empty(),
            Value::Json(obj) => !obj.fields.is_empty(),
            Value::Ncti(n) => {
                let n = n.clone().normalize();
                !(n.limbs.len() == 1 && n.limbs[0] == 0)
            }
            Value::Null => false,
        }
    }

    /// Преобразовать в int (для ncti — только если помещается в один лимб)
    pub fn as_int(&self) -> i64 {
        match self {
            Value::Int(v) => *v,
            Value::Float(v) => *v as i64,
            Value::Bool(v) => if *v { 1 } else { 0 },
            Value::Ncti(n) => n
                .to_i64()
                .unwrap_or_else(|| panic!("ncti-число слишком велико для int")),
            _ => panic!("Нельзя преобразовать в int"),
        }
    }

    /// Преобразовать в float
    pub fn as_float(&self) -> f64 {
        match self {
            Value::Float(v) => *v,
            Value::Int(v) => *v as f64,
            Value::Bool(v) => if *v { 1.0 } else { 0.0 },
            Value::Null => 0.0,
            _ => panic!("Нельзя преобразовать в float"),
        }
    }

    /// Преобразовать в ncti (обычное число автоматически "упаковывается" в один лимб)
    pub fn as_ncti(&self) -> Ncti {
        match self {
            Value::Ncti(n) => n.clone(),
            Value::Int(v) => Ncti::from_i64(*v),
            Value::Bool(v) => Ncti::from_i64(if *v { 1 } else { 0 }),
            Value::Null => Ncti::zero(),
            _ => panic!("Нельзя преобразовать в ncti"),
        }
    }

    /// Преобразовать в String
    pub fn as_string(&self) -> String {
        match self {
            Value::String(v) => v.clone(),
            Value::Int(v) => v.to_string(),
            Value::Float(v) => v.to_string(),
            Value::Bool(v) => v.to_string(),
            Value::Ncti(n) => n.to_decimal_string(),
            Value::Null => "null".to_string(),
            Value::Array(v) => format!("{:?}", v),
            Value::Json(_) => self.to_string(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
            Value::Ncti(n) => write!(f, "{}", n.to_decimal_string()),
            Value::Null => write!(f, "null"),
            Value::String(v) => write!(f, "{}", v),
            Value::Array(values) => {
                write!(f, "[")?;
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", value)?;
                }
                write!(f, "]")
            }
            Value::Json(obj) => {
                write!(f, "{{")?;
                for (i, (key, field)) in obj.fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, field.value)?;
                }
                write!(f, "}}")
            }
        }
    }
}
