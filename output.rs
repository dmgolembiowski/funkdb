#![feature(prelude_import)]
#![allow(unused_imports, non_snake_case, non_camel_case_types, dead_code)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use anyhow::{self as ah, anyhow, bail, Error, Result};
use std::borrow::Cow;
use std::cell::{RefCell, RefMut};
use std::collections::BTreeMap;
use std::fs::{self, File};
#[cfg(any(unix, target_os = "wasi"))]
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::thread;
use strum::{EnumIter, IntoEnumIterator};
use typed_builder::TypedBuilder;
struct Module<'a> {
    name: Cow<'a, str>,
    interner: Rc<RefCell<Interner<'a>>>,
}
#[automatically_derived]
impl<'a> ::core::fmt::Debug for Module<'a> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "Module",
            "name",
            &self.name,
            "interner",
            &&self.interner,
        )
    }
}
#[automatically_derived]
impl<'a> ::core::clone::Clone for Module<'a> {
    #[inline]
    fn clone(&self) -> Module<'a> {
        Module {
            name: ::core::clone::Clone::clone(&self.name),
            interner: ::core::clone::Clone::clone(&self.interner),
        }
    }
}
impl<'a> Module<'a> {
    #[doc(hidden)]
    pub fn new(name: Cow<'a, str>, interner: Rc<RefCell<Interner<'a>>>) -> Self {
        Self { name, interner }
    }
    #[doc(hidden)]
    pub fn builder() -> ModuleBuilder<'a> {
        ModuleBuilder::new()
    }
    pub fn get_name(&self) -> Cow<'a, str> {
        self.name.clone()
    }
    #[allow(unreachable_code)]
    fn add_type(&mut self, _type: FunkTy<'a>) -> anyhow::Result<()> {
        return ::anyhow::__private::Err({
            let error = ::anyhow::__private::format_err(format_args!("Not implemented"));
            error
        });
        {
            ::core::panicking::panic_fmt(
                format_args!(
                    "not yet implemented: {0}",
                    format_args!("Define an `InternerEntry` that can be stored and later retrieved")
                ),
            );
        };
        {
            ::core::panicking::panic_fmt(
                format_args!(
                    "not yet implemented: {0}",
                    format_args!("Encode `r#type`\'s metadata as a bytestream.")
                ),
            );
        };
        {
            ::core::panicking::panic_fmt(
                format_args!(
                    "not yet implemented: {0}",
                    format_args!("Commit the new metadata into the Interner")
                ),
            );
        };
    }
}
#[doc(hidden)]
struct ModuleBuilder<'a> {
    name: Option<Cow<'a, str>>,
    interner: Option<Rc<RefCell<Interner<'a>>>>,
}
#[automatically_derived]
impl<'a> ::core::fmt::Debug for ModuleBuilder<'a> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "ModuleBuilder",
            "name",
            &self.name,
            "interner",
            &&self.interner,
        )
    }
}
#[automatically_derived]
impl<'a> ::core::clone::Clone for ModuleBuilder<'a> {
    #[inline]
    fn clone(&self) -> ModuleBuilder<'a> {
        ModuleBuilder {
            name: ::core::clone::Clone::clone(&self.name),
            interner: ::core::clone::Clone::clone(&self.interner),
        }
    }
}
#[doc(hidden)]
impl<'a> ModuleBuilder<'a> {
    pub fn new() -> Self {
        Self { name: None, interner: None }
    }
    fn build(self) -> Module<'a> {
        let Self { name, interner } = self;
        let name = name.unwrap_or(Cow::from("default"));
        let interner = {
            if let Some(thing) = interner {
                thing
            } else {
                Rc::new(RefCell::new(Interner::new()))
            }
        };
        Module::new(name, interner)
    }
    fn name<T: Into<Cow<'a, str>>>(self, new_name: T) -> Self {
        Self {
            name: Some(new_name.into()),
            interner: self.interner,
        }
    }
    fn interner(self, new_interner: Rc<RefCell<Interner<'a>>>) -> Self {
        Self {
            name: self.name,
            interner: Some(new_interner),
        }
    }
}
struct Namespace<'a> {
    #[builder]
    interner: Rc<RefCell<Interner<'a>>>,
    #[builder]
    modules: Vec<&'a mut Module<'a>>,
}
#[automatically_derived]
impl<'a> Namespace<'a> {
    /**
                Create a builder for building `Namespace`.
                On the builder, call `.interner(...)`, `.modules(...)` to set the values of the fields.
                Finally, call `.build()` to create the instance of `Namespace`.
                */
    #[allow(dead_code, clippy::default_trait_access)]
    fn builder() -> NamespaceBuilder<'a, ((), ())> {
        NamespaceBuilder {
            fields: ((), ()),
            phantom: ::core::default::Default::default(),
        }
    }
}
#[must_use]
#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, non_snake_case)]
struct NamespaceBuilder<'a, TypedBuilderFields = ((), ())> {
    fields: TypedBuilderFields,
    phantom: ::core::marker::PhantomData<(&'a ())>,
}
#[automatically_derived]
impl<'a, TypedBuilderFields> Clone for NamespaceBuilder<'a, TypedBuilderFields>
where
    TypedBuilderFields: Clone,
{
    #[allow(clippy::default_trait_access)]
    fn clone(&self) -> Self {
        Self {
            fields: self.fields.clone(),
            phantom: ::core::marker::PhantomData,
        }
    }
}
#[allow(dead_code, non_camel_case_types, missing_docs)]
#[automatically_derived]
impl<'a, __modules> NamespaceBuilder<'a, ((), __modules)> {
    #[allow(clippy::used_underscore_binding)]
    pub fn interner(
        self,
        interner: Rc<RefCell<Interner<'a>>>,
    ) -> NamespaceBuilder<'a, ((Rc<RefCell<Interner<'a>>>,), __modules)> {
        let interner = (interner,);
        let ((), modules) = self.fields;
        NamespaceBuilder {
            fields: (interner, modules),
            phantom: self.phantom,
        }
    }
}
#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, non_snake_case)]
#[allow(clippy::exhaustive_enums)]
pub enum NamespaceBuilder_Error_Repeated_field_interner {}
#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, missing_docs)]
#[automatically_derived]
impl<'a, __modules> NamespaceBuilder<'a, ((Rc<RefCell<Interner<'a>>>,), __modules)> {
    #[deprecated(note = "Repeated field interner")]
    pub fn interner(
        self,
        _: NamespaceBuilder_Error_Repeated_field_interner,
    ) -> NamespaceBuilder<'a, ((Rc<RefCell<Interner<'a>>>,), __modules)> {
        self
    }
}
#[allow(dead_code, non_camel_case_types, missing_docs)]
#[automatically_derived]
impl<'a, __interner> NamespaceBuilder<'a, (__interner, ())> {
    #[allow(clippy::used_underscore_binding)]
    pub fn modules(
        self,
        modules: Vec<&'a mut Module<'a>>,
    ) -> NamespaceBuilder<'a, (__interner, (Vec<&'a mut Module<'a>>,))> {
        let modules = (modules,);
        let (interner, ()) = self.fields;
        NamespaceBuilder {
            fields: (interner, modules),
            phantom: self.phantom,
        }
    }
}
#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, non_snake_case)]
#[allow(clippy::exhaustive_enums)]
pub enum NamespaceBuilder_Error_Repeated_field_modules {}
#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, missing_docs)]
#[automatically_derived]
impl<'a, __interner> NamespaceBuilder<'a, (__interner, (Vec<&'a mut Module<'a>>,))> {
    #[deprecated(note = "Repeated field modules")]
    pub fn modules(
        self,
        _: NamespaceBuilder_Error_Repeated_field_modules,
    ) -> NamespaceBuilder<'a, (__interner, (Vec<&'a mut Module<'a>>,))> {
        self
    }
}
#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, non_snake_case)]
#[allow(clippy::exhaustive_enums)]
pub enum NamespaceBuilder_Error_Missing_required_field_interner {}
#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, missing_docs, clippy::panic)]
#[automatically_derived]
impl<'a, __modules> NamespaceBuilder<'a, ((), __modules)> {
    #[deprecated(note = "Missing required field interner")]
    pub fn build(self, _: NamespaceBuilder_Error_Missing_required_field_interner) -> ! {
        ::core::panicking::panic("explicit panic")
    }
}
#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, non_snake_case)]
#[allow(clippy::exhaustive_enums)]
pub enum NamespaceBuilder_Error_Missing_required_field_modules {}
#[doc(hidden)]
#[allow(dead_code, non_camel_case_types, missing_docs, clippy::panic)]
#[automatically_derived]
impl<'a> NamespaceBuilder<'a, ((Rc<RefCell<Interner<'a>>>,), ())> {
    #[deprecated(note = "Missing required field modules")]
    pub fn build(self, _: NamespaceBuilder_Error_Missing_required_field_modules) -> ! {
        ::core::panicking::panic("explicit panic")
    }
}
#[allow(dead_code, non_camel_case_types, missing_docs)]
#[automatically_derived]
impl<
    'a,
> NamespaceBuilder<'a, ((Rc<RefCell<Interner<'a>>>,), (Vec<&'a mut Module<'a>>,))> {
    #[allow(clippy::default_trait_access, clippy::used_underscore_binding)]
    pub fn build(self) -> Namespace<'a> {
        let (interner, modules) = self.fields;
        let interner = interner.0;
        let modules = modules.0;
        #[allow(deprecated)] Namespace { interner, modules }.into()
    }
}
#[automatically_derived]
impl<'a> ::core::fmt::Debug for Namespace<'a> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "Namespace",
            "interner",
            &self.interner,
            "modules",
            &&self.modules,
        )
    }
}
impl<'a> Namespace<'a> {
    fn register_module(&mut self, new_module: &'a mut Module<'a>) -> anyhow::Result<()> {
        if let Some(_found)
            = self
                .modules
                .iter()
                .find(|module| module.get_name() == new_module.get_name())
        {
            return ::anyhow::__private::Err({
                let error = ::anyhow::__private::format_err(
                    format_args!("Module registration occurred twice!"),
                );
                error
            });
        } else {
            self.modules.push(new_module);
            Ok(())
        }
    }
    fn try_commit(
        &mut self,
        _commits: &Vec<(Module<'a>, Vec<FunkData<'a>>)>,
    ) -> anyhow::Result<()> {
        return ::anyhow::__private::Err({
            let error = ::anyhow::__private::format_err(format_args!("Not implemented"));
            error
        });
    }
}
pub(crate) struct Interner<'interner> {
    pub metadata: MetaMap<'interner>,
}
#[automatically_derived]
impl<'interner> ::core::fmt::Debug for Interner<'interner> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "Interner",
            "metadata",
            &&self.metadata,
        )
    }
}
#[automatically_derived]
impl<'interner> ::core::clone::Clone for Interner<'interner> {
    #[inline]
    fn clone(&self) -> Interner<'interner> {
        Interner {
            metadata: ::core::clone::Clone::clone(&self.metadata),
        }
    }
}
pub type MetaMap<'a> = BTreeMap<
    (Option<Cow<'a, str>>, Option<Cow<'a, str>>, Option<Cow<'a, str>>),
    FunkData<'a>,
>;
#[doc(hidden)]
struct Key<'a> {
    r#mod: Option<Cow<'a, str>>,
    identity: Option<Cow<'a, str>>,
    assignment: Option<Cow<'a, str>>,
}
#[automatically_derived]
impl<'a> ::core::default::Default for Key<'a> {
    #[inline]
    fn default() -> Key<'a> {
        Key {
            r#mod: ::core::default::Default::default(),
            identity: ::core::default::Default::default(),
            assignment: ::core::default::Default::default(),
        }
    }
}
impl<'interner> Interner<'interner> {
    pub fn new() -> Self {
        Interner {
            metadata: MetaMap::new(),
        }
    }
    pub fn is_name_available(
        &self,
        module_name: Option<&str>,
        identity: Option<&str>,
        field: Option<&str>,
    ) -> bool {
        match (module_name, identity, field) {
            (Some(module), Some(ident), Some(link_or_prop)) => {
                let k = Key {
                    r#mod: Some(Cow::Borrowed(module)),
                    identity: Some(Cow::Borrowed(ident)),
                    assignment: Some(Cow::Borrowed(link_or_prop)),
                    ..<_>::default()
                };
                self.metadata.get(&(k.r#mod, k.identity, k.assignment)).is_none()
            }
            (Some(module), Some(ident), None) => {
                let k = Key {
                    r#mod: Some(Cow::Borrowed(module)),
                    identity: Some(Cow::Borrowed(ident)),
                    ..<_>::default()
                };
                self.metadata.get(&(k.r#mod, k.identity, k.assignment)).is_none()
            }
            (Some(module), None, None) => {
                let k = Key {
                    r#mod: Some(Cow::Borrowed(module)),
                    ..<_>::default()
                };
                self.metadata.get(&(k.r#mod, k.identity, k.assignment)).is_none()
            }
            _ => {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("Verify the arguments passed to is_name_available"),
                    );
                };
            }
        }
    }
}
pub enum FunkData<'interner> {
    primitive(funkstd),
    custom(FunkTy<'interner>),
}
#[automatically_derived]
impl<'interner> ::core::fmt::Debug for FunkData<'interner> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            FunkData::primitive(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "primitive",
                    &__self_0,
                )
            }
            FunkData::custom(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "custom", &__self_0)
            }
        }
    }
}
#[automatically_derived]
impl<'interner> ::core::clone::Clone for FunkData<'interner> {
    #[inline]
    fn clone(&self) -> FunkData<'interner> {
        match self {
            FunkData::primitive(__self_0) => {
                FunkData::primitive(::core::clone::Clone::clone(__self_0))
            }
            FunkData::custom(__self_0) => {
                FunkData::custom(::core::clone::Clone::clone(__self_0))
            }
        }
    }
}
pub enum funkstd {
    bool,
    int8,
    int16,
    int32,
    int64,
    int128,
    str,
    uint8,
    uint16,
    uint32,
    uint64,
    uint128,
}
///An iterator over the variants of [funkstd]
#[allow(missing_copy_implementations)]
pub struct funkstdIter {
    idx: usize,
    back_idx: usize,
    marker: ::core::marker::PhantomData<()>,
}
impl ::core::fmt::Debug for funkstdIter {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("funkstdIter").field("len", &self.len()).finish()
    }
}
impl funkstdIter {
    fn get(&self, idx: usize) -> Option<funkstd> {
        match idx {
            0usize => ::core::option::Option::Some(funkstd::bool),
            1usize => ::core::option::Option::Some(funkstd::int8),
            2usize => ::core::option::Option::Some(funkstd::int16),
            3usize => ::core::option::Option::Some(funkstd::int32),
            4usize => ::core::option::Option::Some(funkstd::int64),
            5usize => ::core::option::Option::Some(funkstd::int128),
            6usize => ::core::option::Option::Some(funkstd::str),
            7usize => ::core::option::Option::Some(funkstd::uint8),
            8usize => ::core::option::Option::Some(funkstd::uint16),
            9usize => ::core::option::Option::Some(funkstd::uint32),
            10usize => ::core::option::Option::Some(funkstd::uint64),
            11usize => ::core::option::Option::Some(funkstd::uint128),
            _ => ::core::option::Option::None,
        }
    }
}
impl ::strum::IntoEnumIterator for funkstd {
    type Iterator = funkstdIter;
    fn iter() -> funkstdIter {
        funkstdIter {
            idx: 0,
            back_idx: 0,
            marker: ::core::marker::PhantomData,
        }
    }
}
impl Iterator for funkstdIter {
    type Item = funkstd;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        self.nth(0)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let t = if self.idx + self.back_idx >= 12usize {
            0
        } else {
            12usize - self.idx - self.back_idx
        };
        (t, Some(t))
    }
    fn nth(&mut self, n: usize) -> Option<<Self as Iterator>::Item> {
        let idx = self.idx + n + 1;
        if idx + self.back_idx > 12usize {
            self.idx = 12usize;
            ::core::option::Option::None
        } else {
            self.idx = idx;
            self.get(idx - 1)
        }
    }
}
impl ExactSizeIterator for funkstdIter {
    fn len(&self) -> usize {
        self.size_hint().0
    }
}
impl DoubleEndedIterator for funkstdIter {
    fn next_back(&mut self) -> Option<<Self as Iterator>::Item> {
        let back_idx = self.back_idx + 1;
        if self.idx + back_idx > 12usize {
            self.back_idx = 12usize;
            ::core::option::Option::None
        } else {
            self.back_idx = back_idx;
            self.get(12usize - self.back_idx)
        }
    }
}
impl Clone for funkstdIter {
    fn clone(&self) -> funkstdIter {
        funkstdIter {
            idx: self.idx,
            back_idx: self.back_idx,
            marker: self.marker.clone(),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for funkstd {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                funkstd::bool => "bool",
                funkstd::int8 => "int8",
                funkstd::int16 => "int16",
                funkstd::int32 => "int32",
                funkstd::int64 => "int64",
                funkstd::int128 => "int128",
                funkstd::str => "str",
                funkstd::uint8 => "uint8",
                funkstd::uint16 => "uint16",
                funkstd::uint32 => "uint32",
                funkstd::uint64 => "uint64",
                funkstd::uint128 => "uint128",
            },
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for funkstd {
    #[inline]
    fn clone(&self) -> funkstd {
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for funkstd {}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for funkstd {}
#[automatically_derived]
impl ::core::cmp::PartialEq for funkstd {
    #[inline]
    fn eq(&self, other: &funkstd) -> bool {
        let __self_tag = ::core::intrinsics::discriminant_value(self);
        let __arg1_tag = ::core::intrinsics::discriminant_value(other);
        __self_tag == __arg1_tag
    }
}
#[automatically_derived]
impl ::core::marker::StructuralEq for funkstd {}
#[automatically_derived]
impl ::core::cmp::Eq for funkstd {
    #[inline]
    #[doc(hidden)]
    #[no_coverage]
    fn assert_receiver_is_total_eq(&self) -> () {}
}
pub type FunkPropMap<'interner> = BTreeMap<Cow<'interner, str>, (funkstd, bool, bool)>;
pub type FunkLinkMap<'interner> = BTreeMap<
    Cow<'interner, str>,
    (Rc<FunkTy<'interner>>, bool, bool),
>;
pub struct FunkTy<'a> {
    pub type_name: Option<Cow<'a, str>>,
    pub properties: FunkPropMap<'a>,
    pub links: FunkLinkMap<'a>,
}
#[automatically_derived]
impl<'a> ::core::fmt::Debug for FunkTy<'a> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field3_finish(
            f,
            "FunkTy",
            "type_name",
            &self.type_name,
            "properties",
            &self.properties,
            "links",
            &&self.links,
        )
    }
}
#[automatically_derived]
impl<'a> ::core::clone::Clone for FunkTy<'a> {
    #[inline]
    fn clone(&self) -> FunkTy<'a> {
        FunkTy {
            type_name: ::core::clone::Clone::clone(&self.type_name),
            properties: ::core::clone::Clone::clone(&self.properties),
            links: ::core::clone::Clone::clone(&self.links),
        }
    }
}
#[automatically_derived]
impl<'a> ::core::default::Default for FunkTy<'a> {
    #[inline]
    fn default() -> FunkTy<'a> {
        FunkTy {
            type_name: ::core::default::Default::default(),
            properties: ::core::default::Default::default(),
            links: ::core::default::Default::default(),
        }
    }
}
impl<'a> FunkTy<'a> {
    pub fn r#type<T: Into<Cow<'a, str>>>(name: T) -> FunkTy<'a> {
        let mut this = FunkTy::default();
        this.type_name = Some(name.into());
        this
    }
    fn add_property<T: Into<Cow<'a, str>>>(mut self, prop: (T, funkstd)) -> Self {
        let (typekey, property) = prop;
        let required = false;
        let is_multi = false;
        let typekey: Cow<'a, str> = typekey.into();
        self.properties.insert(typekey, (property, required, is_multi));
        self
    }
    fn add_multi_property<T: Into<Cow<'a, str>>>(mut self, prop: (T, funkstd)) -> Self {
        let (typekey, multiproperty) = prop;
        let required = false;
        let is_multi = true;
        let typekey: Cow<'a, str> = typekey.into();
        self.properties.insert(typekey, (multiproperty, required, is_multi));
        self
    }
    fn add_required_property<T: Into<Cow<'a, str>>>(
        mut self,
        prop: (T, funkstd),
    ) -> Self {
        let (typekey, property) = prop;
        let required = true;
        let is_multi = false;
        let typekey: Cow<'a, str> = typekey.into();
        self.properties.insert(typekey, (property, required, is_multi));
        self
    }
    fn add_required_multi_property<T: Into<Cow<'a, str>>>(
        mut self,
        prop: (T, funkstd),
    ) -> Self {
        let (typekey, multiproperty) = prop;
        let required = true;
        let is_multi = true;
        let typekey: Cow<'a, str> = typekey.into();
        self.properties.insert(typekey, (multiproperty, required, is_multi));
        self
    }
    fn add_multi_link<'interner, T: Into<Cow<'a, str>>>(
        mut self,
        link: (T, Rc<FunkTy<'a>>),
    ) -> Self {
        let (linkkey, multilink) = link;
        let required = false;
        let is_multi = true;
        let linkkey: Cow<'a, str> = linkkey.into();
        self.links.insert(linkkey, (multilink, required, is_multi));
        self
    }
    fn add_link<'interner, T: Into<Cow<'a, str>>>(
        mut self,
        link: (T, Rc<FunkTy<'a>>),
    ) -> Self {
        let (linkkey, link) = link;
        let required = false;
        let is_multi = false;
        let linkkey: Cow<'a, str> = linkkey.into();
        self.links.insert(linkkey, (link, required, is_multi));
        self
    }
    fn add_required_multi_link<'interner, T: Into<Cow<'a, str>>>(
        mut self,
        link: (T, Rc<FunkTy<'a>>),
    ) -> Self {
        let (linkkey, multilink) = link;
        let required = true;
        let is_multi = true;
        let linkkey: Cow<'a, str> = linkkey.into();
        self.links.insert(linkkey, (multilink, required, is_multi));
        self
    }
    fn add_required_link<'interner, T: Into<Cow<'a, str>>>(
        mut self,
        link: (T, Rc<FunkTy<'a>>),
    ) -> Self {
        let (linkkey, link) = link;
        let required = true;
        let is_multi = false;
        let linkkey: Cow<'a, str> = linkkey.into();
        self.links.insert(linkkey, (link, required, is_multi));
        self
    }
}
#[allow(dead_code)]
pub struct FunkDb {
    path: PathBuf,
    stream: Option<UnixStream>,
    file: File,
}
impl FunkDb {
    pub fn new<F: IntoRawFd>(path: PathBuf, fileno: Option<F>, file: File) -> Self {
        let stream = match fileno {
            Some(f) => {
                let fd = f.into_raw_fd();
                Some(unsafe { <UnixStream as FromRawFd>::from_raw_fd(fd) })
            }
            None => None,
        };
        Self { path, stream, file }
    }
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = PathBuf::from(path.as_ref());
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        Ok(Self::new(path, Option::<UnixStream>::None, file))
    }
    #[allow(dead_code)]
    fn new_server(
        &mut self,
        server_path: impl AsRef<Path>,
        db_path: impl AsRef<Path>,
    ) -> anyhow::Result<()> {
        let _sfd = FunkDbServer::bind(server_path, db_path)?;
        {
            ::core::panicking::panic_fmt(
                format_args!("not yet implemented: {0}", format_args!("Not implemented")),
            );
        };
    }
    pub fn save(&mut self) -> anyhow::Result<()> {
        if self.stream.is_some() {
            return ::anyhow::__private::Err({
                let error = ::anyhow::__private::format_err(
                    format_args!("Not implemented!"),
                );
                error
            });
        }
        self.file.sync_all()?;
        Ok(())
    }
}
pub struct FunkDbServer {}
impl FunkDbServer {
    /// Returns the result of the bind op
    /// which, assuming the socket path wasn't already taken, should be Ok(i32).
    ///
    /// With the unwrapped return value, the caller can assume
    /// that there is a unix domain socket at [`path`] which
    /// is a [`UnixListener`].
    ///
    /// The listener will be used to accept client connections to the database
    /// so that prepared statements can be executed, queries against the
    /// database can be ran, and transactions to update the schema can be made.
    ///
    /// Note that [`bind`]'s argument, [`path`], is distinct from the actual
    /// database file.
    #[allow(dead_code, unused_variables)]
    pub fn bind(
        server_path: impl AsRef<Path>,
        db_path: impl AsRef<Path>,
    ) -> anyhow::Result<RawFd> {
        let path = server_path.as_ref().to_string_lossy();
        let stream = db_path.as_ref().to_string_lossy();
        let server = UnixListener::bind(db_path)?.set_nonblocking(true);
        return ::anyhow::__private::Err({
            let error = ::anyhow::__private::format_err(
                format_args!("This is not yet implemented"),
            );
            error
        });
    }
}
