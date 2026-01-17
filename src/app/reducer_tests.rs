#[cfg(test)]
mod tests {
    use crate::app::{
        action::Action,
        reducer::reduce_action,
        screen::Screen,
        state::{AppState, ConnectionItem},
    };

    fn mk_state_with_connections(names: &[&str]) -> AppState {
        let mut state = AppState::new();
        state.screen = Screen::Connections;

        state.connections.items = names
            .iter()
            .enumerate()
            .map(|(i, &name)| {
                // Make each connection distinct but deterministic enough.
                ConnectionItem::new(
                    name,
                    "localhost",
                    3306 + (i as u16),
                    "root",
                    "pw",
                    "db",
                )
            })
            .collect();

        state.connections.selected = 0;
        state
    }

    #[test]
    fn delete_selected_opens_confirmation_modal() {
        let mut state = mk_state_with_connections(&["a", "b"]);
        state.connections.selected = 1;

        let cmds = reduce_action(&mut state, Action::DeleteSelectedConnection);
        assert!(cmds.is_empty(), "opening confirm modal should emit no commands");

        let confirm = state
            .connections
            .delete_confirm
            .as_ref()
            .expect("delete_confirm should be set");

        assert_eq!(confirm.name, "b");
        assert_eq!(confirm.id, state.connections.items[1].id);
        assert!(
            state.status.message.contains("Delete connection"),
            "should surface a prompt in status"
        );
    }

    #[test]
    fn delete_cancel_clears_confirmation_modal_and_keeps_items() {
        let mut state = mk_state_with_connections(&["a", "b"]);
        state.connections.selected = 0;

        let _ = reduce_action(&mut state, Action::DeleteSelectedConnection);
        assert!(state.connections.delete_confirm.is_some());

        let cmds = reduce_action(&mut state, Action::CancelDeleteConnection);
        assert!(cmds.is_empty(), "cancel should emit no commands");
        assert!(state.connections.delete_confirm.is_none());

        assert_eq!(state.connections.items.len(), 2);
        assert_eq!(state.connections.items[0].name, "a");
        assert_eq!(state.connections.items[1].name, "b");
        assert_eq!(state.status.message, "Delete cancelled");
    }

    #[test]
    fn delete_confirm_removes_by_id_and_emits_save_command() {
        let mut state = mk_state_with_connections(&["a", "b", "c"]);
        state.connections.selected = 1;
        let id_to_delete = state.connections.items[1].id;

        let _ = reduce_action(&mut state, Action::DeleteSelectedConnection);
        assert!(state.connections.delete_confirm.is_some());

        let cmds = reduce_action(&mut state, Action::ConfirmDeleteConnection);

        // Expect a save command to be emitted so storage worker persists removal.
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            crate::app::command::Command::Storage(
                crate::app::command::StorageCommand::SaveConnections { connections },
            ) => {
                // Persisted profiles should exclude the deleted item.
                assert_eq!(connections.len(), 2);
                assert!(connections.iter().all(|p| p.id != id_to_delete.to_string()));
            }
            other => panic!("unexpected command emitted: {:?}", other),
        }

        // State should be updated.
        assert_eq!(state.connections.items.len(), 2);
        assert_eq!(state.connections.items[0].name, "a");
        assert_eq!(state.connections.items[1].name, "c");
        assert!(state.connections.items.iter().all(|c| c.id != id_to_delete));
        assert!(state.connections.delete_confirm.is_none());
    }

    #[test]
    fn delete_confirm_when_already_deleted_is_noop() {
        let mut state = mk_state_with_connections(&["a", "b"]);
        state.connections.selected = 0;

        // Open confirm on "a"
        let _ = reduce_action(&mut state, Action::DeleteSelectedConnection);
        let confirm = state.connections.delete_confirm.clone().expect("confirm open");

        // Simulate connection removed by some other path before confirming
        state
            .connections
            .items
            .retain(|c| c.id != confirm.id);

        let cmds = reduce_action(&mut state, Action::ConfirmDeleteConnection);
        assert!(cmds.is_empty(), "should not emit save command when nothing removed");
        assert_eq!(state.status.message, "Connection already deleted");
    }

    #[test]
    fn edit_selected_opens_prefilled_modal() {
        let mut state = mk_state_with_connections(&["a", "b"]);
        state.connections.selected = 0;

        let cmds = reduce_action(&mut state, Action::EditSelectedConnection);
        assert!(cmds.is_empty(), "opening edit modal should emit no commands");

        let draft = state.connections.adding.as_ref().expect("draft should be set");
        assert!(
            draft.is_edit,
            "edit_from should set is_edit so UI can reflect edit mode"
        );
        assert_eq!(draft.name, "a");
        assert_eq!(draft.host, "localhost");
        assert_eq!(draft.user, "root");
        assert_eq!(draft.database, "db");
    }

    #[test]
    fn edit_when_no_selection_sets_status_message() {
        let mut state = AppState::new();
        state.screen = Screen::Connections;
        state.connections.items.clear();
        state.connections.selected = 0;

        let cmds = reduce_action(&mut state, Action::EditSelectedConnection);
        assert!(cmds.is_empty());
        assert_eq!(state.status.message, "No connection selected");
        assert!(state.connections.adding.is_none());
    }
}
