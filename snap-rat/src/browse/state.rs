use crate::app::{App, SortMode};

impl App {
    pub fn toggle_installed_filter(&mut self) {
        self.show_installed_only = !self.show_installed_only;
        self.list_state.select(Some(0));
    }

    pub fn cycle_sort(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::Relevance => SortMode::NameAsc,
            SortMode::NameAsc => SortMode::NameDesc,
            SortMode::NameDesc => SortMode::RevisionDesc,
            SortMode::RevisionDesc => SortMode::Relevance,
        };
        self.list_state.select(Some(0));
    }

    pub fn next(&mut self) {
        let len = self.display_snaps().len();
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1).min(len - 1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn prev(&mut self) {
        let len = self.display_snaps().len();
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(0) | None => 0,
            Some(i) => i - 1,
        };
        self.list_state.select(Some(i));
    }

    pub fn page_down(&mut self) {
        let len = self.display_snaps().len();
        if len == 0 {
            return;
        }
        let i = self
            .list_state
            .selected()
            .unwrap_or(0)
            .saturating_add(10)
            .min(len - 1);
        self.list_state.select(Some(i));
    }

    pub fn page_up(&mut self) {
        let len = self.display_snaps().len();
        if len == 0 {
            return;
        }
        let i = self.list_state.selected().unwrap_or(0).saturating_sub(10);
        self.list_state.select(Some(i));
    }
}
