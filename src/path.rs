use std::{
    fmt::{self, Debug},
    iter::FromIterator,
    ops::Deref,
};

#[repr(transparent)]
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct ModulePath(str);

impl ModulePath {
    pub fn new(s: &str) -> &Self {
        unsafe { &*(s as *const str as *const ModulePath) }
    }

    pub fn into_module_path_buf(&self) -> ModulePathBuf {
        self.0.to_string().into()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn is_crate_path(&self) -> bool {
        self.starts_with("crate")
    }

    pub fn starts_with(&self, base: impl AsRef<ModulePath>) -> bool {
        self.0.starts_with(&base.as_ref().0)
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter(self)
    }

    pub fn join(&self, suffix: impl AsRef<ModulePath>) -> ModulePathBuf {
        let mut res = self.into_module_path_buf();
        res.push(suffix);
        res
    }

    pub fn suffix(&self) -> &Self {
        if let Some(i) = self.0.find("::") {
            Self::new(&self.0[i + 2..])
        } else {
            Self::new("")
        }
    }
}

impl<'a> IntoIterator for &'a ModulePath {
    type Item = &'a str;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        (*self).iter()
    }
}

impl AsRef<ModulePath> for ModulePath {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<ModulePath> for ModulePathBuf {
    fn as_ref(&self) -> &ModulePath {
        self.0.as_ref()
    }
}

impl AsRef<ModulePath> for str {
    fn as_ref(&self) -> &ModulePath {
        ModulePath::new(self)
    }
}

impl AsRef<ModulePath> for String {
    fn as_ref(&self) -> &ModulePath {
        self.as_str().as_ref()
    }
}

impl Debug for ModulePathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct ModulePathBuf(String);

impl ModulePathBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn as_module_path(&self) -> &ModulePath {
        ModulePath::new(self.0.as_str())
    }

    pub fn push(&mut self, suffix: impl AsRef<ModulePath>) {
        let suffix = suffix.as_ref();
        if suffix.starts_with("::") || suffix.starts_with("crate")
        /* || suffix.starts_with("self") */
        {
            self.0.clear();
        }
        if !self.is_empty() {
            self.0.reserve(2 + suffix.len());
            self.0.push_str("::");
        }
        self.0.push_str(&suffix.0);
    }

    pub fn pop(&mut self) -> bool {
        if let Some(del) = self.0.rfind("::") {
            self.0.truncate(del);
            true
        } else {
            false
        }
    }
}

impl Deref for ModulePathBuf {
    type Target = ModulePath;

    fn deref(&self) -> &Self::Target {
        self.as_module_path()
    }
}

impl From<&ModulePath> for ModulePathBuf {
    fn from(path: &ModulePath) -> Self {
        path.into_module_path_buf()
    }
}

impl From<String> for ModulePathBuf {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ModulePathBuf {
    fn from(s: &str) -> Self {
        ModulePath::new(s).into_module_path_buf()
    }
}

impl<P: AsRef<ModulePath>> Extend<P> for ModulePathBuf {
    fn extend<I: IntoIterator<Item = P>>(&mut self, iter: I) {
        for p in iter {
            self.push(p);
        }
    }
}

impl<P: AsRef<ModulePath>> FromIterator<P> for ModulePathBuf {
    fn from_iter<I: IntoIterator<Item = P>>(iter: I) -> Self {
        let mut res = Self::new();
        res.extend(iter);
        res
    }
}

pub struct Iter<'a>(&'a ModulePath);

impl<'a> Iter<'a> {
    pub fn as_path(&self) -> &'a ModulePath {
        self.0
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((res, tail)) = self.0.as_str().split_once("::") {
            self.0 = ModulePath::new(tail);
            Some(res)
        } else {
            None
        }
    }
}
