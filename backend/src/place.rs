use std::iter;

use tokio::sync::RwLock;

type Pixel = RwLock<(u8, u8, u8)>;

type Row = RwLock<Vec<Pixel>>;

type Grid = RwLock<Vec<Row>>;

const WHITE: (u8, u8, u8) = (255, 255, 255);

pub struct State {
    pub grid: Grid,
    current_grid_size: (usize, usize),
}
impl State {
    pub fn new() -> Self {
        State {
            grid: RwLock::new(Vec::new()),
            current_grid_size: (0, 0),
        }
    }

    pub async fn set_grid_size(&mut self, width: usize, height: usize) {
        if (width, height) == self.current_grid_size {
            return;
        }
        // let delta_width = width.sub;
    }

    pub async fn add_rows(&mut self, number_of_rows: usize) {
        let mut grid_guard = self.grid.write().await;
        grid_guard.append(
            // Repeat for each row
            &mut iter::repeat_with(|| {
                RwLock::new(
                    // Repeat for each column
                    iter::repeat_with(|| RwLock::new(WHITE))
                        .take(self.current_grid_size.0)
                        .collect(),
                )
            })
            .take(number_of_rows)
            .collect(),
        );
        self.current_grid_size.1 += number_of_rows;
    }

    pub async fn add_columns(&mut self, number_of_columns: usize) {
        let grid_guard = self.grid.read().await;
        for row_lock in grid_guard.iter() {
            let mut row_guard = row_lock.write().await;
            row_guard.append(
                &mut iter::repeat_with(|| RwLock::new(WHITE))
                    .take(number_of_columns)
                    .collect(),
            );
        }
        self.current_grid_size.0 += number_of_columns;
    }

    pub async fn remove_rows(&mut self, number_of_rows: usize) {
        let mut grid_guard = self.grid.write().await;
        let final_number_of_rows = self.current_grid_size.1.saturating_sub(number_of_rows);
        grid_guard.truncate(final_number_of_rows);
        self.current_grid_size.1 = final_number_of_rows;
    }

    pub async fn remove_columns(&mut self, number_of_columns: usize) {
        let grid_guard = self.grid.read().await;
        let final_number_of_columns = self.current_grid_size.0.saturating_sub(number_of_columns);
        for row_lock in grid_guard.iter() {
            let mut row_guard = row_lock.write().await;
            row_guard.truncate(final_number_of_columns);
        }
        self.current_grid_size.0 = final_number_of_columns;
    }
}
