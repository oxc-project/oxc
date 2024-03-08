use std::str::FromStr;

//
// JSDocTypeExpression
//

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamTypeKind {
    Any,
    Repeated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParamType<'a> {
    pub value: &'a str,
}

impl<'a> ParamType<'a> {
    #[allow(unused)]
    pub fn kind(&self) -> Option<ParamTypeKind> {
        ParamTypeKind::from_str(self.value).map(Option::Some).unwrap_or_default()
    }
}

impl FromStr for ParamTypeKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: This might be inaccurate if the type is listed as {....string} or some variant
        if s.len() > 3 && &s[0..3] == "..." {
            return Ok(Self::Repeated);
        }

        if s == "*" {
            return Ok(Self::Any);
        }

        Err(())
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Param<'a> {
    pub name: &'a str,
    pub r#type: Option<ParamType<'a>>,
}

//
// Structs
//

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JSDocTagKind<'a> {
    Access,
    Deprecated,
    Package,
    Parameter(Param<'a>),
    Private,
    Protected,
    Public,
    Unknown(&'a str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JSDocTag<'a> {
    pub kind: JSDocTagKind<'a>,
    pub comment: String,
}

impl<'a> JSDocTag<'a> {
    pub fn tag_name(&self) -> &'a str {
        match self.kind {
            JSDocTagKind::Access => "access",
            JSDocTagKind::Deprecated => "deprecated",
            JSDocTagKind::Package => "package",
            JSDocTagKind::Parameter(_) => "param",
            JSDocTagKind::Private => "private",
            JSDocTagKind::Protected => "protected",
            JSDocTagKind::Public => "public",
            JSDocTagKind::Unknown(tag_name) => tag_name,
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Param, ParamType, ParamTypeKind};

    #[test]
    fn deduces_correct_param_kind() {
        let param = Param { name: "a", r#type: Some(ParamType { value: "string" }) };
        assert_eq!(param.r#type.and_then(|t| t.kind()), None);

        let param = Param { name: "a", r#type: Some(ParamType { value: "...string" }) };
        assert_eq!(param.r#type.and_then(|t| t.kind()), Some(ParamTypeKind::Repeated));

        let param = Param { name: "a", r#type: Some(ParamType { value: "*" }) };
        assert_eq!(param.r#type.and_then(|t| t.kind()), Some(ParamTypeKind::Any));
    }
}
