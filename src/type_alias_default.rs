// TODO: remove
/// Same as the [`Default`] trait.
///
/// The only different is that it is defined in this crate, and so it can be implemented
/// for type aliases (type aliases to not belong to the create, thus they are orphans).
/// https://doc.rust-lang.org/book/ch10-02-traits.html
pub trait TypeAliasDefault {
    fn default() -> Self;
}
