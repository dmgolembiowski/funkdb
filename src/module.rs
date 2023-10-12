use anyhow::{self as ah, anyhow, bail, Error, Result};
use std::borrow::{Cow, BorrowMut};
use std::cell::{RefCell, RefMut};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::fs::{self, File};
#[cfg(any(unix, target_os = "wasi"))]
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::thread;
use strum::{EnumIter, IntoEnumIterator};
use typed_builder::TypedBuilder;
use crate::{FunkData, FunkTy};
use crate::namespace::Namespace;
use std::rc::Weak;


// Note: Module<'a> automatically implements ToOwned and Borrow<Self<'_>>
// 
// `module default { ... }`
#[derive(Debug, Clone)]
pub(crate) struct Module<'ns> {
    pub(crate) module_name: Box<*const Cow<'ns, str>>,
    pub(crate) module_member_entries: Vec<Member<'ns>>,
}

#[derive(Debug, Clone)]
pub(crate) struct Member<'ns> {
    pub(crate) member_name: String,
    pub(crate) fields: Option<Vec<MemberData<'ns>>>,
    pub(crate) attrs: Option<AttributeMap>,
}

// ```
// module default { 
//      type A; /* <- This is a member */
//      type B { /* <- another member */
//        ...
//      }
// }
// ```
#[derive(Debug, Clone)]
pub(crate) struct  MemberField {
    pub(crate) field_name: String,
    pub(crate) layout: Option<Vec<MemberLayout>>,
}

/// QualPath is the complete 
/// name used to describe the data type 
/// for a given property or link 
/// on a member's field.
///
/// If the type exists outside of the current
/// module-level, then either a top-level `using`
/// statement must be present. Otherwise, the fully-qualified
/// syntax is required to resolve the link/property. 
trait QualPath: Debug {}

#[derive(Debug, Clone)]
pub(crate) enum MemberData<'ns> {
    LocalMember(MemberField),
    Nonlocal(&'ns dyn QualPath),
    Std(&'ns dyn QualPath),
}
// ```
// module default { 
//      type A; 
//      type B {
//        required title: str;
//        
//      }
// }
// ```/ 
#[derive(Debug, Clone)]
pub(crate) struct MemberLayout {
    identity: String,
    attribute_map: AttributeMap,
}

#[derive(Debug, Clone)]
pub(crate) struct AttributeMap {
    pub(crate) is_required: bool,
    pub(crate) is_multi: bool,
    pub(crate) is_link: bool,
    pub(crate) constraint: Option<Constraint>,
    pub(crate) extends: Option<Vec</* some abstract type ... maybe generic trait? */ ()>>,
}

#[derive(Debug, Clone)]
pub(crate) struct Constraint;

impl<'a> Module<'a> {
    #[doc(hidden)]
    pub fn new<'b>(module_name: Rc<Cow<'a, str>>, entries: Option<Vec<Member<'b>>>) -> Self {
        let name_ref = Rc::clone(&module_name);
        let module_name: Box<*const Cow<'a, str>>  = Box::new(Rc::downgrade(&name_ref).into_raw());
        let module_member_entries = entries.unwrap_or(vec![]);
        Self { module_name, module_member_entries }
    }
    #[doc(hidden)]
    pub fn builder() -> ModuleBuilder<'a> {
        ModuleBuilder::new()
    }
    pub fn get_name(&self) -> &Cow<'a, str> {
        let raw: *const Cow<'a, str> = *self.module_name.clone();
        let module_name: Option<Rc<_>> = unsafe { Weak::from_raw(raw) }.upgrade();
        assert!(module_name.is_some());
        let module_name = module_name.unwrap();
        &*module_name
    }
    #[allow(unreachable_code)]
    fn add_type(&mut self, _type: FunkTy<'a>) -> anyhow::Result<()> {
        // Inspect the interner for the presented `r#type: FunkTy<'a>`
        // metadata. If an associated `InternerEntry` is found, we
        // yeet an error back to the caller.
        bail!("`add_type` Not implemented");
        todo!("Define an `InternerEntry` that can be stored and later retrieved");
        todo!("Encode `r#type`'s metadata as a bytestream.");
        todo!("Commit the new metadata into the Interner");
    }
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub(crate) struct ModuleBuilder<'a> {
    module_name: Option<Cow<'a, str>>,
    module_member_entries: Option<Vec<Member<'a>>>,
}

#[doc(hidden)]
impl<'a> ModuleBuilder<'a> {
    pub fn new() -> Self {
        Self {
            module_name: None,
            module_member_entries: None,
        }
    }
    fn build(self) -> Module<'a> {
        let Self { module_name, module_member_entries } = self;
        let module_name = module_name.unwrap_or(Cow::from(String::from("default")));
        let module_name = Rc::new(module_name);
        Module::new(module_name, module_member_entries)
    }
    fn module_name<T: Into<Cow<'a, str>>>(mut self, new_name: T) -> Self {
        let Self { module_name, module_member_entries } = self;
        Self {
            module_name: Some(new_name.into()),
            module_member_entries,
        }
    }
}
