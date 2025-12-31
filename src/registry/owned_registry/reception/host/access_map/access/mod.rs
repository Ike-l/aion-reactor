pub trait Access {
    /// Can both `self` and `other` coexist?
    fn can_coexit(&self, other: &Self) -> bool;

    fn access();
}