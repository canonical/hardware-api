use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Attribute, Expr, ExprPath, Ident, LitStr, Path, Result, Token,
};

pub mod kw {
    syn::custom_keyword!(annotation);
    syn::custom_keyword!(attribute);
    syn::custom_keyword!(cancel_handle);
    syn::custom_keyword!(dict);
    syn::custom_keyword!(extends);
    syn::custom_keyword!(freelist);
    syn::custom_keyword!(from_py_with);
    syn::custom_keyword!(frozen);
    syn::custom_keyword!(get);
    syn::custom_keyword!(get_all);
    syn::custom_keyword!(item);
    syn::custom_keyword!(from_item_all);
    syn::custom_keyword!(mapping);
    syn::custom_keyword!(module);
    syn::custom_keyword!(name);
    syn::custom_keyword!(pass_module);
    syn::custom_keyword!(rename_all);
    syn::custom_keyword!(sequence);
    syn::custom_keyword!(set);
    syn::custom_keyword!(set_all);
    syn::custom_keyword!(signature);
    syn::custom_keyword!(subclass);
    syn::custom_keyword!(text_signature);
    syn::custom_keyword!(transparent);
    syn::custom_keyword!(unsendable);
    syn::custom_keyword!(weakref);
}

#[derive(Clone, Debug)]
pub struct KeywordAttribute<K, V> {
    pub kw: K,
    pub value: V,
}

/// A helper type which parses the inner type via a literal string
/// e.g. `LitStrValue<Path>` -> parses "some::path" in quotes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LitStrValue<T>(pub T);

impl<T: Parse> Parse for LitStrValue<T> {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lit_str: LitStr = input.parse()?;
        lit_str.parse().map(LitStrValue)
    }
}

impl<T: ToTokens> ToTokens for LitStrValue<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

/// A helper type which parses a name via a literal string
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameLitStr(pub Ident);

impl Parse for NameLitStr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let string_literal: LitStr = input.parse()?;
        if let Ok(ident) = string_literal.parse() {
            Ok(NameLitStr(ident))
        } else {
            bail_spanned!(string_literal.span() => "expected a single identifier in double quotes")
        }
    }
}

impl ToTokens for NameLitStr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

/// Available renaming rules
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenamingRule {
    CamelCase,
    KebabCase,
    Lowercase,
    PascalCase,
    ScreamingKebabCase,
    ScreamingSnakeCase,
    SnakeCase,
    Uppercase,
}

/// A helper type which parses a renaming rule via a literal string
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenamingRuleLitStr {
    pub lit: LitStr,
    pub rule: RenamingRule,
}

impl Parse for RenamingRuleLitStr {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let string_literal: LitStr = input.parse()?;
        let rule = match string_literal.value().as_ref() {
            "camelCase" => RenamingRule::CamelCase,
            "kebab-case" => RenamingRule::KebabCase,
            "lowercase" => RenamingRule::Lowercase,
            "PascalCase" => RenamingRule::PascalCase,
            "SCREAMING-KEBAB-CASE" => RenamingRule::ScreamingKebabCase,
            "SCREAMING_SNAKE_CASE" => RenamingRule::ScreamingSnakeCase,
            "snake_case" => RenamingRule::SnakeCase,
            "UPPERCASE" => RenamingRule::Uppercase,
            _ => {
                bail_spanned!(string_literal.span() => "expected a valid renaming rule, possible values are: \"camelCase\", \"kebab-case\", \"lowercase\", \"PascalCase\", \"SCREAMING-KEBAB-CASE\", \"SCREAMING_SNAKE_CASE\", \"snake_case\", \"UPPERCASE\"")
            }
        };
        Ok(Self {
            lit: string_literal,
            rule,
        })
    }
}

impl ToTokens for RenamingRuleLitStr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.lit.to_tokens(tokens)
    }
}

/// Text signatue can be either a literal string or opt-in/out
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextSignatureAttributeValue {
    Str(LitStr),
    // `None` ident to disable automatic text signature generation
    Disabled(Ident),
}

impl Parse for TextSignatureAttributeValue {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if let Ok(lit_str) = input.parse::<LitStr>() {
            return Ok(TextSignatureAttributeValue::Str(lit_str));
        }

        let err_span = match input.parse::<Ident>() {
            Ok(ident) if ident == "None" => {
                return Ok(TextSignatureAttributeValue::Disabled(ident));
            }
            Ok(other_ident) => other_ident.span(),
            Err(e) => e.span(),
        };

        Err(err_spanned!(err_span => "expected a string literal or `None`"))
    }
}

impl ToTokens for TextSignatureAttributeValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            TextSignatureAttributeValue::Str(s) => s.to_tokens(tokens),
            TextSignatureAttributeValue::Disabled(b) => b.to_tokens(tokens),
        }
    }
}

pub type ExtendsAttribute = KeywordAttribute<kw::extends, Path>;
pub type FreelistAttribute = KeywordAttribute<kw::freelist, Box<Expr>>;
pub type ModuleAttribute = KeywordAttribute<kw::module, LitStr>;
pub type NameAttribute = KeywordAttribute<kw::name, NameLitStr>;
pub type RenameAllAttribute = KeywordAttribute<kw::rename_all, RenamingRuleLitStr>;
pub type TextSignatureAttribute = KeywordAttribute<kw::text_signature, TextSignatureAttributeValue>;

impl<K: Parse + std::fmt::Debug, V: Parse> Parse for KeywordAttribute<K, V> {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let kw: K = input.parse()?;
        let _: Token![=] = input.parse()?;
        let value = input.parse()?;
        Ok(KeywordAttribute { kw, value })
    }
}

impl<K: ToTokens, V: ToTokens> ToTokens for KeywordAttribute<K, V> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.kw.to_tokens(tokens);
        Token![=](self.kw.span()).to_tokens(tokens);
        self.value.to_tokens(tokens);
    }
}

pub type FromPyWithAttribute = KeywordAttribute<kw::from_py_with, LitStrValue<ExprPath>>;

/// For specifying the path to the pyo3 crate.
pub type CrateAttribute = KeywordAttribute<Token![crate], LitStrValue<Path>>;

pub fn get_pyo3_options<T: Parse>(attr: &syn::Attribute) -> Result<Option<Punctuated<T, Comma>>> {
    if attr.path().is_ident("pyo3") {
        attr.parse_args_with(Punctuated::parse_terminated).map(Some)
    } else {
        Ok(None)
    }
}

/// Takes attributes from an attribute vector.
///
/// For each attribute in `attrs`, `extractor` is called. If `extractor` returns `Ok(true)`, then
/// the attribute will be removed from the vector.
///
/// This is similar to `Vec::retain` except the closure is fallible and the condition is reversed.
/// (In `retain`, returning `true` keeps the element, here it removes it.)
pub fn take_attributes(
    attrs: &mut Vec<Attribute>,
    mut extractor: impl FnMut(&Attribute) -> Result<bool>,
) -> Result<()> {
    *attrs = attrs
        .drain(..)
        .filter_map(|attr| {
            extractor(&attr)
                .map(move |attribute_handled| if attribute_handled { None } else { Some(attr) })
                .transpose()
        })
        .collect::<Result<_>>()?;
    Ok(())
}

pub fn take_pyo3_options<T: Parse>(attrs: &mut Vec<syn::Attribute>) -> Result<Vec<T>> {
    let mut out = Vec::new();
    take_attributes(attrs, |attr| {
        if let Some(options) = get_pyo3_options(attr)? {
            out.extend(options);
            Ok(true)
        } else {
            Ok(false)
        }
    })?;
    Ok(out)
}
