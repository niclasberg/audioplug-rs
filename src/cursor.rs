#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Cursor {
    Arrow,
    Hand,
    Help,
    IBeam,
    NotAllowed,
    Wait,
    Cross,
    UpDown,
    LeftRight,
    LeftUpRightDown,
    LeftDownRightUp,    
}