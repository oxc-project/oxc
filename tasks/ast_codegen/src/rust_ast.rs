use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use syn::{
    braced,
    parse::{Parse, ParseBuffer},
    parse_quote,
    punctuated::Punctuated,
    Attribute, Generics, Ident, Item, ItemEnum, ItemMacro, ItemStruct, Meta, Path, Token, Type,
    Variant, Visibility,
};

use crate::{
    layout::Layout,
    util::{unexpanded_macro_err, NormalizeError},
};

use super::{parse_file, Itertools, PathBuf, Rc, Read, RefCell, Result};

pub type AstRef = Rc<RefCell<AstType>>;

#[derive(Debug, Clone)]
pub enum Inherit {
    Unlinked(String),
    Linked { super_: Type, variants: Punctuated<Variant, Token![,]> },
}

impl From<Ident> for Inherit {
    fn from(ident: Ident) -> Self {
        Self::Unlinked(ident.to_string())
    }
}

#[derive(Debug, Default, Clone)]
pub struct EnumMeta {
    pub inherits: Vec<Inherit>,
    pub layout_32: Layout,
    pub layout_64: Layout,
    pub visitable: bool,
    pub ast: bool,
}

#[derive(Debug)]
pub struct Enum {
    pub item: ItemEnum,
    pub meta: EnumMeta,
}

impl Enum {
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

impl From<ItemEnum> for Enum {
    fn from(item: ItemEnum) -> Self {
        Self { item, meta: EnumMeta::default() }
    }
}

/// Placeholder for now!
#[derive(Debug, Default, Clone)]
pub struct StructMeta {
    pub layout_32: Layout,
    pub layout_64: Layout,
    pub visitable: bool,
    pub ast: bool,
}

#[derive(Debug)]
pub struct Struct {
    pub item: ItemStruct,
    pub meta: StructMeta,
}

impl Struct {
    pub fn ident(&self) -> &Ident {
        &self.item.ident
    }

    pub fn as_type(&self) -> Type {
        let ident = self.ident();
        let generics = &self.item.generics;
        parse_quote!(#ident #generics)
    }
}

impl From<ItemStruct> for Struct {
    fn from(item: ItemStruct) -> Self {
        Self { item, meta: StructMeta::default() }
    }
}

#[derive(Debug)]
pub enum AstType {
    Enum(Enum),
    Struct(Struct),

    // we need this to expand `inherit` macro calls.
    Macro(ItemMacro),
}

impl ToTokens for AstType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Enum(it) => it.item.to_tokens(tokens),
            Self::Struct(it) => it.item.to_tokens(tokens),

            Self::Macro(it) => it.to_tokens(tokens),
        }
    }
}

impl AstType {
    pub fn ident(&self) -> Option<&Ident> {
        match self {
            AstType::Enum(ty) => Some(ty.ident()),
            AstType::Struct(ty) => Some(ty.ident()),
            AstType::Macro(tt) => tt.ident.as_ref(),
        }
    }

    pub fn as_type(&self) -> Option<Type> {
        match self {
            AstType::Enum(it) => Some(it.as_type()),
            AstType::Struct(it) => Some(it.as_type()),
            AstType::Macro(_) => None,
        }
    }

    pub fn visitable(&self) -> bool {
        match self {
            AstType::Enum(it) => it.meta.visitable,
            AstType::Struct(it) => it.meta.visitable,
            AstType::Macro(_) => false,
        }
    }

    pub fn set_visitable(&mut self, value: bool) -> Result<()> {
        macro_rules! assign {
            ($it:ident) => {{
                debug_assert!($it.meta.ast, "only ast types can be visitable!");
                $it.meta.visitable = value;
            }};
        }
        match self {
            AstType::Enum(it) => assign!(it),
            AstType::Struct(it) => assign!(it),
            AstType::Macro(it) => return Err(unexpanded_macro_err(it)),
        }
        Ok(())
    }

    pub fn set_ast(&mut self, value: bool) -> Result<()> {
        match self {
            AstType::Enum(it) => it.meta.ast = value,
            AstType::Struct(it) => it.meta.ast = value,
            AstType::Macro(it) => return Err(unexpanded_macro_err(it)),
        }
        Ok(())
    }

    pub fn layout_32(&self) -> Result<Layout> {
        match self {
            AstType::Enum(it) => Ok(it.meta.layout_32.clone()),
            AstType::Struct(it) => Ok(it.meta.layout_32.clone()),
            AstType::Macro(it) => Err(unexpanded_macro_err(it)),
        }
    }

    pub fn layout_64(&self) -> Result<Layout> {
        match self {
            AstType::Enum(it) => Ok(it.meta.layout_64.clone()),
            AstType::Struct(it) => Ok(it.meta.layout_64.clone()),
            AstType::Macro(it) => Err(unexpanded_macro_err(it)),
        }
    }

    pub fn layouts(&self) -> Result<(/* 64 */ Layout, /* 32 */ Layout)> {
        self.layout_64().and_then(|x64| self.layout_32().map(|x32| (x64, x32)))
    }

    pub fn set_layout(&mut self, layout_64: Layout, layout_32: Layout) -> Result<()> {
        macro_rules! assign {
            ($it:ident) => {{
                $it.meta.layout_32 = layout_32;
                $it.meta.layout_64 = layout_64;
            }};
        }
        match self {
            AstType::Enum(it) => assign!(it),
            AstType::Struct(it) => assign!(it),
            AstType::Macro(it) => return Err(unexpanded_macro_err(it)),
        }
        Ok(())
    }
}

impl TryFrom<Item> for AstType {
    type Error = String;
    fn try_from(item: Item) -> Result<Self> {
        match item {
            Item::Enum(it) => Ok(AstType::Enum(it.into())),
            Item::Struct(it) => Ok(AstType::Struct(it.into())),
            Item::Macro(it) => Ok(AstType::Macro(it)),
            _ => Err(String::from("Unsupported Item!")),
        }
    }
}

const LOAD_ERROR: &str = "should be loaded by now!";
#[derive(Debug)]
pub struct Module {
    pub path: PathBuf,
    // TODO: remove me
    #[allow(dead_code)]
    #[allow(clippy::struct_field_names)]
    pub module: String,
    pub shebang: Option<String>,
    pub attrs: Vec<Attribute>,
    pub items: Vec<AstRef>,
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

        let mut file = std::fs::File::open(&self.path).normalize()?;
        let mut content = String::new();
        file.read_to_string(&mut content).normalize()?;
        let file = parse_file(content.as_str()).normalize()?;
        self.shebang = file.shebang;
        self.attrs = file.attrs;
        self.items = file
            .items
            .into_iter()
            .filter(|it| match it {
                Item::Enum(_) | Item::Struct(_) => true,
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
}

pub fn expand(ast_ref: &AstRef) -> Result<()> {
    let to_replace = match &*ast_ref.borrow() {
        AstType::Macro(mac) => {
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
                .normalize()?;
            Some(AstType::Enum(Enum::with_meta(
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
        *ast_ref.borrow_mut() = to_replace;
    }

    Ok(())
}

pub fn analyze(ast_ref: &AstRef) -> Result<()> {
    enum AstAttr {
        None,
        Mark,
        Visit,
    }
    let ast_attr = match &*ast_ref.borrow() {
        AstType::Enum(Enum { item: ItemEnum { attrs, .. }, .. })
        | AstType::Struct(Struct { item: ItemStruct { attrs, .. }, .. }) => {
            let attr = attrs.iter().find(|attr| attr.path().is_ident("ast"));
            let attr = match attr {
                Some(Attribute { meta: Meta::Path(_), .. }) => AstAttr::Mark,
                Some(attr @ Attribute { meta: Meta::List(_), .. }) => {
                    // TODO: support for punctuated list of arguments here if needed!
                    let args = attr.parse_args::<Path>().normalize()?;
                    if args.is_ident("visit") {
                        AstAttr::Visit
                    } else {
                        AstAttr::Mark
                    }
                }
                Some(_) => return Err(String::from("Invalid arguments in the `ast` attribute!")),
                None => AstAttr::None,
            };
            Some(attr)
        }
        AstType::Macro(_) => None,
    };

    #[allow(clippy::match_same_arms)]
    match ast_attr {
        Some(AstAttr::Visit) => {
            ast_ref.borrow_mut().set_ast(true)?;
            ast_ref.borrow_mut().set_visitable(true)?;
        }
        Some(AstAttr::Mark) => {
            // AST without visit!
            ast_ref.borrow_mut().set_ast(true)?;
        }
        Some(AstAttr::None) => return Err(format!("All `enums` and `structs` defined in the source of truth should be marked with an `#[ast]` attribute(missing `#[ast]` on '{:?}')", ast_ref.borrow().ident())),
        None => { /* unrelated items like `use`, `type` and `macro` definitions */ }
    }

    Ok(())
}

impl From<PathBuf> for Module {
    fn from(path: PathBuf) -> Self {
        Self::with_path(path)
    }
}
