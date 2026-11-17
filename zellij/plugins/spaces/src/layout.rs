#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Geometry {
    pub rows: usize,
    pub cols: usize,
    pub search_y: Option<usize>,
    pub list_top: usize,
    pub list_height: usize,
    pub detail_y: Option<usize>,
    pub hints_y: usize,
}

pub fn geometry(rows: usize, cols: usize) -> Geometry {
    let search_y = if rows >= 10 { Some(1) } else { None };
    let list_top = if search_y.is_some() { 3 } else { 1 };
    let hints_y = rows.saturating_sub(2);
    let detail_y = if rows >= 16 {
        Some(rows.saturating_sub(4))
    } else {
        None
    };
    let list_bottom = if let Some(detail_y) = detail_y {
        detail_y.saturating_sub(1)
    } else {
        hints_y.saturating_sub(1)
    };
    Geometry {
        rows,
        cols,
        search_y,
        list_top,
        list_height: list_bottom.saturating_sub(list_top),
        detail_y,
        hints_y,
    }
}

pub fn window_start(selected: usize, total: usize, height: usize) -> usize {
    if height == 0 || total <= height {
        return 0;
    }
    let centered = selected.saturating_sub(height / 2);
    centered.min(total - height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_layout_has_room_for_search_list_detail_and_hints() {
        let geometry = geometry(30, 100);
        assert_eq!(geometry.search_y, Some(1));
        assert_eq!(geometry.list_top, 3);
        assert_eq!(geometry.detail_y, Some(26));
        assert_eq!(geometry.hints_y, 28);
        assert!(geometry.list_height > 10);
    }

    #[test]
    fn short_layout_drops_the_detail_line_before_the_list() {
        let geometry = geometry(14, 80);
        assert_eq!(geometry.detail_y, None);
        assert_eq!(geometry.hints_y, 12);
        assert!(geometry.list_height > 0);
    }

    #[test]
    fn the_window_follows_the_selection_and_stops_at_the_end() {
        assert_eq!(window_start(0, 30, 10), 0);
        assert_eq!(window_start(5, 30, 10), 0);
        assert_eq!(window_start(10, 30, 10), 5);
        assert_eq!(window_start(29, 30, 10), 20);
        assert_eq!(window_start(3, 8, 10), 0);
    }
}
