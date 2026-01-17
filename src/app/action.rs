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

    // connection
    OpenAddConnection,
    DeleteSelectedConnection,

    // delete-confirm modal
    ConfirmDeleteConnection,
    CancelDeleteConnection,

    // add-connection modal
    CancelModal,
    NextField,
    PrevField,
    Backspace,
    InputChar(char),

    // app control
    Quit,
}
