use super::Type;

impl From<&str> for Type {
    fn from(value: &str) -> Self {
        match value {
            "text" => Self::Text,
            "char" => Self::Character,
            "bool" => Self::Bool,
            "int" => Self::Int,
            "int8" => Self::Int8,
            "int16" => Self::Int16,
            "int32" => Self::Int32,
            "int64" => Self::Int64,
            "uint" => Self::UInt,
            "uint8" => Self::UInt8,
            "uint16" => Self::UInt16,
            "uint32" => Self::UInt32,
            "uint64" => Self::UInt64,
            "float" => Self::Float,
            "float32" => Self::Float32,
            "float64" => Self::Float64,
            _ => Self::Struct(value.into()),
        }
    }
}

impl From<String> for Type {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}
