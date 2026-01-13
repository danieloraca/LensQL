#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Back,

    // screen jumps
    GoConnections,
    GoSchema,
    GoData,
    GoQueries,
    GoRunner,

    // actions
    ConnectSelected,
    Disconnect,

    // app control
    Quit,
}
