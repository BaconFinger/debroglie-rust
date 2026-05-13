/// Specifies the way in which a tile can be symmetric.
/// The letters are chosen that they have the letter itself
/// has the symmetry group it represents.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TileSymmetry {

    /// No symmetry
    F = 0,

    /// Fully symmetric
    X,

    /// Reflectable on y-axis
    T,

    /// Reflectable on x-axis and y-axis
    I,

    /// Reflectable on one diagonal
    L,

    /// Reflectable on both diagonals.
    Slash,

    /// Reflectable on other diagonal
    Q,

    /// Can rotate 180 degrees
    N,

    /// Reflectable on x-axis
    E,

    /// Any rotation, but no reflection.
    /// There's no keyboard symbol that corresponds to this!
    Cyclic
}

impl Default for TileSymmetry {
    fn default() -> Self {
        TileSymmetry::F
    }
}

impl TileSymmetry {
    fn none() -> Self {
        TileSymmetry::default()
    }

    pub fn parse(s: &str) -> Option<TileSymmetry> {
        match s.to_uppercase().as_str() {
            "F" => {Some(TileSymmetry::F)}
            "X" => {Some(TileSymmetry::X)}
            "T" => {Some(TileSymmetry::T)}
            "I" => {Some(TileSymmetry::I)}
            "L" => {Some(TileSymmetry::L)}
            "/" => {Some(TileSymmetry::Slash)}
            "\\" => {Some(TileSymmetry::Slash)}
            "N" => {Some(TileSymmetry::N)}
            "E" => {Some(TileSymmetry::E)}
            "Q" => {Some(TileSymmetry::Q)}
            "NONE" => {Some(TileSymmetry::none())}
            "FULL" => {Some(TileSymmetry::X)}
            "CYCLIC" => {Some(TileSymmetry::Cyclic)}
            _ => None
        }
    }
}