#[derive(Clone, Copy)]
pub enum PathField {
    EnumType,
    MessageType,
    Service,
    Field,
    Method,
    EnumValue,
    NestedType,
    NestedEnum,
}

impl PathField {
    pub fn value(self) -> i32 {
        match self {
            PathField::EnumType => 5,
            PathField::MessageType | PathField::NestedEnum => 4,
            PathField::Service => 6,
            PathField::Field | PathField::Method | PathField::EnumValue => 2,
            PathField::NestedType => 3,
        }
    }
}
