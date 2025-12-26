// "Foo|FooBarAdapter|Bar|BarBazAdapter|Baz", FooInput
// "FooBarAdapter|Bar|BarBazAdapter|Baz", FooOutput
// "Bar|BarBazAdapter|Baz", BarInput 
// "BarBazAdapter|Baz", BarOutput 
// "Baz", BazInput
// Complete

use crate::prelude::ExecutableMessage;

pub struct QueuedExecutable {
    pub label: String,
    pub message: ExecutableMessage
}

impl QueuedExecutable {
    pub fn new(label: String, message: ExecutableMessage) -> Self {
        Self { label, message }
    }
}

#[derive(Default)]
pub struct ExecutableQueue(Vec<QueuedExecutable>);

impl ExecutableQueue {
    pub fn queue(&mut self, executable: QueuedExecutable) {
        self.0.push(executable);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = QueuedExecutable> {
        self.0.drain(..)
    }

    pub fn extend<T>(&mut self, iter: T) 
        where T: IntoIterator<Item = QueuedExecutable> 
    {
        self.0.extend(iter);
    }
}
