//! Player focus for the right-hand detail column and hit testing.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SquadId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Selection {
    #[default]
    None,
    Base,
    Mission,
    Squad(SquadId),
}
