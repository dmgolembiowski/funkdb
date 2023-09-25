use anyhow::{self as ah, anyhow, bail, Error, Result};
use std::borrow::{Cow, BorrowMut};
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

use crate::namespace::Namespace;
use std::rc::Weak;


// Note: Module<'a> automatically implements ToOwned and Borrow<Self<'_>>
// 
// `module default { ... }`
#[derive(Debug, Clone)]
pub(crate) struct Module<'a> {
    module_name: Box<*const Cow<'a, str>>,
    module_member_entries: Vec<(
        /* field_name = */ String,
        (
            /* `module_name::module_member_identity` alias */ String,
            /* attribute_map = */ AttributeMap,
        )
    )>,
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
pub(crate) struct  Member {
    field_name: String,
    layout: Option<Vec<MemberLayout>>,
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
    is_required: bool,
    is_multi: bool,
    is_link: bool,
    constraint: Option<Constraint>,
    extends: Option<Vec</* some abstract type ... maybe generic trait? */ ()>>,
}

#[derive(Debug, Clone)]
pub(crate) struct Constraint;

impl<'a> Module<'a> {
    #[doc(hidden)]
    pub fn new(module_name: &Rc<Cow<'a, str>>, module_member_entries: &) -> Self {
        let name_ref = Rc::clone(module_name);
        let module_name: Box<*const Cow<'a, str>>  = Box::new(Rc::downgrade(&name_ref).into_raw());
        Self { module_name }
    }
    #[doc(hidden)]
    pub fn builder() -> ModuleBuilder<'a> {
        ModuleBuilder::new()
    }
    pub fn get_name(&self) -> &Cow<'a, str> {
        &self.name
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
struct ModuleBuilder<'a> {
    name: Option<Cow<'a, str>>,
    interner: Option<Rc<RefCell<Interner<'a>>>>,
}

#[doc(hidden)]
impl<'a> ModuleBuilder<'a> {
    pub fn new() -> Self {
        Self {
            name: None,
            interner: None,
        }
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
