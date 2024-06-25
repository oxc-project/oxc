use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use syn::{
    braced,
    parse::{Parse, ParseBuffer},
    parse_quote,
    punctuated::Punctuated,
    Attribute, Generics, Ident, Item, ItemConst, ItemEnum, ItemMacro, ItemStruct, ItemUse, Token,
    Type, Variant, Visibility,
};

use crate::TypeName;

use super::{parse_file, Itertools, PathBuf, Rc, Read, RefCell, Result, TypeDef, TypeRef};

#[derive(Debug, serde::Serialize)]
pub struct Schema {
    source: PathBuf,
    definitions: Definitions,
}

#[derive(Debug, serde::Serialize)]
pub struct Definitions {
    types: Vec<TypeDef>,
}

#[derive(Debug, Clone)]
pub enum Inherit {
    Unlinked(String),
    Linked { super_: String, variants: Punctuated<Variant, Token![,]> },
}

impl From<Ident> for Inherit {
    fn from(ident: Ident) -> Self {
        Self::Unlinked(ident.to_string())
    }
}

#[derive(Debug, Default, Clone)]
pub struct EnumMeta {
    pub inherits: Vec<Inherit>,
    pub visitable: bool,
}

#[derive(Debug)]
pub struct REnum {
    pub item: ItemEnum,
    pub meta: EnumMeta,
}

impl REnum {
    pub fn with_meta(item: ItemEnum, meta: EnumMeta) -> Self {
        Self { item, meta }
    }

    pub fn ident(&self) -> &Ident {
        &self.item.ident
    }

    pub fn as_type(&self) -> Type {
        let ident = self.ident();
        let generics = &self.item.generics;
        parse_quote!(#ident #generics)
    }
}

impl From<ItemEnum> for REnum {
    fn from(item: ItemEnum) -> Self {
        Self { item, meta: EnumMeta::default() }
    }
}

/// Placeholder for now!
#[derive(Debug, Default, Clone)]
pub struct StructMeta {
    pub visitable: bool,
}

#[derive(Debug)]
pub struct RStruct {
    pub item: ItemStruct,
    pub meta: StructMeta,
}

impl RStruct {
    pub fn ident(&self) -> &Ident {
        &self.item.ident
    }

    pub fn as_type(&self) -> Type {
        let ident = self.ident();
        let generics = &self.item.generics;
        parse_quote!(#ident #generics)
    }
}

impl From<ItemStruct> for RStruct {
    fn from(item: ItemStruct) -> Self {
        Self { item, meta: StructMeta::default() }
    }
}

#[derive(Debug)]
pub enum RType {
    Enum(REnum),
    Struct(RStruct),

    Use(ItemUse),
    Const(ItemConst),
    Macro(ItemMacro),
}

impl ToTokens for RType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Enum(it) => it.item.to_tokens(tokens),
            Self::Struct(it) => it.item.to_tokens(tokens),

            Self::Use(it) => it.to_tokens(tokens),
            Self::Const(it) => it.to_tokens(tokens),
            Self::Macro(it) => it.to_tokens(tokens),
        }
    }
}

impl RType {
    pub fn ident(&self) -> Option<&Ident> {
        match self {
            RType::Enum(ty) => Some(ty.ident()),
            RType::Struct(ty) => Some(ty.ident()),

            RType::Use(_) => None,
            RType::Macro(tt) => tt.ident.as_ref(),
            RType::Const(tt) => Some(&tt.ident),
        }
    }

    pub fn as_type(&self) -> Option<Type> {
        match self {
            RType::Enum(it) => Some(it.as_type()),
            RType::Struct(it) => Some(it.as_type()),
            _ => None,
        }
    }

    pub fn visitable(&self) -> bool {
        match self {
            RType::Enum(it) => it.meta.visitable,
            RType::Struct(it) => it.meta.visitable,
            _ => false,
        }
    }

    pub fn set_visitable(&mut self, value: bool) -> Result<()> {
        match self {
            RType::Enum(it) => it.meta.visitable = value,
            RType::Struct(it) => it.meta.visitable = value,
            _ => return Err("Unsupported type!".to_string()),
        }
        Ok(())
    }
}

impl TryFrom<Item> for RType {
    type Error = String;
    fn try_from(item: Item) -> Result<Self> {
        match item {
            Item::Enum(it) => Ok(RType::Enum(it.into())),
            Item::Struct(it) => Ok(RType::Struct(it.into())),
            Item::Macro(it) => Ok(RType::Macro(it)),
            Item::Use(it) => Ok(RType::Use(it)),
            Item::Const(it) => Ok(RType::Const(it)),
            _ => Err(String::from("Unsupported Item!")),
        }
    }
}

const LOAD_ERROR: &str = "should be loaded by now!";
#[derive(Debug)]
pub struct Module {
    pub path: PathBuf,
    #[allow(clippy::struct_field_names)]
    pub module: TypeName,
    pub shebang: Option<String>,
    pub attrs: Vec<Attribute>,
    pub items: Vec<TypeRef>,
    pub loaded: bool,
}

impl ToTokens for Module {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.attrs.clone());
        self.items.iter().for_each(|it| it.borrow().to_tokens(tokens));
    }
}

impl Module {
    pub fn with_path(path: PathBuf) -> Self {
        let module = path.file_stem().map(|it| it.to_string_lossy().to_string()).unwrap();
        Self { path, module, shebang: None, attrs: Vec::new(), items: Vec::new(), loaded: false }
    }

    pub fn load(mut self) -> Result<Self> {
        assert!(!self.loaded, "can't load twice!");

        let mut file = std::fs::File::open(&self.path).map_err(|e| e.to_string())?;
        let mut content = String::new();
        file.read_to_string(&mut content).map_err(|e| e.to_string())?;
        let file = parse_file(content.as_str()).map_err(|e| e.to_string())?;
        self.shebang = file.shebang;
        self.attrs = file.attrs;
        self.items = file
            .items
            .into_iter()
            .filter(|it| match it {
                Item::Enum(_) | Item::Struct(_) | Item::Use(_) | Item::Const(_) => true,
                // These contain enums with inheritance
                Item::Macro(m) if m.mac.path.is_ident("inherit_variants") => true,
                _ => false,
            })
            .map(TryInto::try_into)
            .map_ok(|it| Rc::new(RefCell::new(it)))
            .collect::<Result<_>>()?;
        self.loaded = true;
        Ok(self)
    }

    /// Expand `inherit_variants` macros to their inner enum.
    /// This would also populate `inherits` field of `EnumMeta` types.
    pub fn expand(self) -> Result<Self> {
        if !self.loaded {
            return Err(String::from(LOAD_ERROR));
        }

        self.items.iter().try_for_each(expand)?;
        Ok(self)
    }

    /// Fills the Meta types.
    pub fn analyze(self) -> Result<Self> {
        if !self.loaded {
            return Err(String::from(LOAD_ERROR));
        }

        self.items.iter().try_for_each(analyze)?;
        Ok(self)
    }

    pub fn build(self) -> Result<Schema> {
        if !self.loaded {
            return Err(String::from(LOAD_ERROR));
        }

        let definitions = Definitions {
            // We filter map to get rid of stuff we don't need in our schema.
            types: self.items.into_iter().filter_map(|it| (&*it.borrow()).into()).collect(),
        };
        Ok(Schema { source: self.path, definitions })
    }
}

pub fn expand(type_def: &TypeRef) -> Result<()> {
    let to_replace = match &*type_def.borrow() {
        RType::Macro(mac) => {
            let (enum_, inherits) = mac
                .mac
                .parse_body_with(|input: &ParseBuffer| {
                    // Because of `@inherit`s we can't use the actual `ItemEnum` parse,
                    // This closure is similar to how `ItemEnum` parser works, With the exception
                    // of how we approach our variants, First we try to parse a variant out of our
                    // tokens if we fail we try parsing the inheritance, And we would raise an
                    // error only if both of these fail.
                    let attrs = input.call(Attribute::parse_outer)?;
                    let vis = input.parse::<Visibility>()?;
                    let enum_token = input.parse::<Token![enum]>()?;
                    let ident = input.parse::<Ident>()?;
                    let generics = input.parse::<Generics>()?;
                    let (where_clause, brace_token, variants, inherits) = {
                        let where_clause = input.parse()?;

                        let content;
                        let brace = braced!(content in input);
                        let mut variants = Punctuated::new();
                        let mut inherits = Vec::<Ident>::new();
                        while !content.is_empty() {
                            if let Ok(variant) = Variant::parse(&content) {
                                variants.push_value(variant);
                                let punct = content.parse()?;
                                variants.push_punct(punct);
                            } else if content.parse::<Token![@]>().is_ok()
                                && content.parse::<Ident>().is_ok_and(|id| id == "inherit")
                            {
                                inherits.push(content.parse::<Ident>()?);
                            } else {
                                panic!("Invalid inherit_variants usage!");
                            }
                        }

                        (where_clause, brace, variants, inherits)
                    };
                    Ok((
                        ItemEnum {
                            attrs,
                            vis,
                            enum_token,
                            ident,
                            generics: Generics { where_clause, ..generics },
                            brace_token,
                            variants,
                        },
                        inherits,
                    ))
                })
                .map_err(|e| e.to_string())?;
            Some(RType::Enum(REnum::with_meta(
                enum_,
                EnumMeta {
                    inherits: inherits.into_iter().map(Into::into).collect(),
                    ..EnumMeta::default()
                },
            )))
        }
        _ => None,
    };

    if let Some(to_replace) = to_replace {
        *type_def.borrow_mut() = to_replace;
    }

    Ok(())
}

pub fn analyze(type_def: &TypeRef) -> Result<()> {
    let is_visitable = match &*type_def.borrow() {
        RType::Enum(REnum { item: ItemEnum { attrs, .. }, .. })
        | RType::Struct(RStruct { item: ItemStruct { attrs, .. }, .. }) => {
            Some(attrs.iter().any(|attr| attr.path().is_ident("visited_node")))
        }
        _ => None,
    };

    if let Some(is_visitable) = is_visitable {
        type_def.borrow_mut().set_visitable(is_visitable)?;
    }

    Ok(())
}

impl From<PathBuf> for Module {
    fn from(path: PathBuf) -> Self {
        Self::with_path(path)
    }
}
