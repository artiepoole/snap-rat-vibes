use std::time::Instant;

use snapd_rs_artie::Change;

use crate::app::App;

impl App {
    pub fn changes_next(&mut self) {
        let len = self.changes_list.len();
        if len == 0 {
            return;
        }
        let i = match self.changes_list_state.selected() {
            Some(i) => (i + 1).min(len - 1),
            None => 0,
        };
        self.changes_list_state.select(Some(i));
        self.changes_detail_state.select(Some(0));
    }

    pub fn changes_prev(&mut self) {
        let len = self.changes_list.len();
        if len == 0 {
            return;
        }
        let i = match self.changes_list_state.selected() {
            Some(0) | None => 0,
            Some(i) => i - 1,
        };
        self.changes_list_state.select(Some(i));
        self.changes_detail_state.select(Some(0));
    }

    pub fn changes_detail_next(&mut self) {
        let len = self
            .selected_change()
            .map(|change| change.tasks.len())
            .unwrap_or(0);
        if len == 0 {
            return;
        }
        let i = match self.changes_detail_state.selected() {
            Some(i) => (i + 1).min(len - 1),
            None => 0,
        };
        self.changes_detail_state.select(Some(i));
    }

    pub fn changes_detail_prev(&mut self) {
        let len = self
            .selected_change()
            .map(|change| change.tasks.len())
            .unwrap_or(0);
        if len == 0 {
            return;
        }
        let i = match self.changes_detail_state.selected() {
            Some(0) | None => 0,
            Some(i) => i - 1,
        };
        self.changes_detail_state.select(Some(i));
    }

    pub fn selected_change(&self) -> Option<&Change> {
        let idx = self.changes_list_state.selected()?;
        self.changes_list.get(idx)
    }

    pub async fn poll_sidebar_changes(&mut self) {
        if let Ok(changes) = self.client.list_changes().await {
            self.sidebar_changes = changes;
        }
    }

    pub async fn load_changes(&mut self) {
        self.loading = true;
        self.error = None;
        match self.client.list_all_changes().await {
            Ok(mut changes) => {
                sort_changes(&mut changes);
                self.changes_list = changes;
                self.changes_focus_detail = false;
                self.changes_list_state
                    .select((!self.changes_list.is_empty()).then_some(0));
                self.changes_detail_state.select(
                    self.selected_change()
                        .and_then(|change| (!change.tasks.is_empty()).then_some(0)),
                );
            }
            Err(e) => {
                self.error = Some(e.to_string());
            }
        }
        self.changes_last_polled = Some(Instant::now());
        self.loading = false;
    }
    pub async fn poll_changes(&mut self) {
        if let Ok(mut changes) = self.client.list_all_changes().await {
            let selected_id = self
                .changes_list_state
                .selected()
                .and_then(|i| self.changes_list.get(i))
                .map(|c| c.id.clone());
            sort_changes(&mut changes);
            self.changes_list = changes;
            // Restore selection by id, fall back to same index, else first item.
            let new_idx = selected_id
                .and_then(|id| self.changes_list.iter().position(|c| c.id == id))
                .or(self.changes_list_state.selected())
                .map(|i| i.min(self.changes_list.len().saturating_sub(1)))
                .filter(|_| !self.changes_list.is_empty());
            self.changes_list_state.select(new_idx);
        }
        self.changes_last_polled = Some(Instant::now());
    }

    pub async fn abort_selected_change(&mut self) {
        let Some(change) = self.selected_change().cloned() else {
            return;
        };
        if change.ready {
            self.status_message = Some("Change already finished".to_string());
            return;
        }

        self.loading = true;
        self.error = None;
        match self.client.abort_change(&change.id).await {
            Ok(_) => {
                self.status_message = Some("Abort requested".to_string());
                self.load_changes().await;
            }
            Err(e) => {
                self.error = Some(e.to_string());
            }
        }
        self.loading = false;
    }
}

fn sort_changes(changes: &mut [Change]) {
    changes.sort_by(|a, b| {
        // In-progress before finished
        let a_active = !a.ready;
        let b_active = !b.ready;
        b_active.cmp(&a_active).then_with(|| {
            // Most recent first
            b.spawn_time
                .as_deref()
                .unwrap_or("")
                .cmp(a.spawn_time.as_deref().unwrap_or(""))
        })
    });
}
