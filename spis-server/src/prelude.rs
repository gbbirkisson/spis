use std::{
    ffi::OsStr,
    path::{Component, PathBuf},
};

use tokio::sync::mpsc::Receiver;

pub struct W<T>(pub T);

pub struct RecieverIterator<T> {
    reciever: Receiver<T>,
}

impl<T> Iterator for RecieverIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.reciever.blocking_recv()
    }
}

impl<T> IntoIterator for W<Receiver<T>> {
    type Item = T;
    type IntoIter = RecieverIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter { reciever: self.0 }
    }
}

impl From<W<PathBuf>> for String {
    fn from(value: W<PathBuf>) -> Self {
        value.0.display().to_string()
    }
}

impl From<W<&PathBuf>> for String {
    fn from(value: W<&PathBuf>) -> Self {
        value.0.display().to_string()
    }
}

impl From<W<&OsStr>> for String {
    fn from(value: W<&OsStr>) -> Self {
        value
            .0
            .to_str()
            .expect("Failed to get String from OsStr")
            .to_string()
    }
}

impl From<W<Component<'_>>> for String {
    fn from(value: W<Component>) -> Self {
        Self::from(W(value.0.as_os_str()))
    }
}
