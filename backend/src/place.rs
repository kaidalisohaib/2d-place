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
            current_grid_size: (0, 0), // (width, height)
        }
    }

    pub fn get_grid_width(&self) -> usize {
        return self.current_grid_size.0;
    }
    pub fn get_grid_height(&self) -> usize {
        return self.current_grid_size.0;
    }

    pub async fn read_grid(&self) -> Vec<Vec<(u8, u8, u8)>> {
        let mut cloned_grid: Vec<Vec<(u8, u8, u8)>> = Vec::with_capacity(self.get_grid_height());
        for row_lock in self.grid.read().await.iter() {
            let mut cloned_row: Vec<(u8, u8, u8)> = Vec::with_capacity(self.get_grid_width());
            for pixel_lock in row_lock.read().await.iter() {
                let pixel_guard = pixel_lock.read().await;
                cloned_row.push(*pixel_guard);
            }
            cloned_grid.push(cloned_row);
        }
        cloned_grid
    }

    pub async fn set_grid_size(&mut self, new_width: usize, new_height: usize) {
        // Add/Remove rows
        if new_height > self.current_grid_size.1 {
            self.add_rows(new_height - self.current_grid_size.1).await;
        } else if new_height < self.current_grid_size.1 {
            self.remove_rows(self.current_grid_size.1 - new_height)
                .await;
        }

        // Add/Remove columns
        if new_width > self.current_grid_size.0 {
            self.add_columns(new_width - self.current_grid_size.0).await;
        } else if new_height < self.current_grid_size.1 {
            self.remove_columns(self.current_grid_size.0 - new_width)
                .await;
        }
    }

    async fn add_rows(&mut self, number_of_rows: usize) {
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

    async fn add_columns(&mut self, number_of_columns: usize) {
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

    async fn remove_rows(&mut self, number_of_rows: usize) {
        let mut grid_guard = self.grid.write().await;
        let final_number_of_rows = self.current_grid_size.1.saturating_sub(number_of_rows);
        grid_guard.truncate(final_number_of_rows);
        self.current_grid_size.1 = final_number_of_rows;
    }

    async fn remove_columns(&mut self, number_of_columns: usize) {
        let grid_guard = self.grid.read().await;
        let final_number_of_columns = self.current_grid_size.0.saturating_sub(number_of_columns);
        for row_lock in grid_guard.iter() {
            let mut row_guard = row_lock.write().await;
            row_guard.truncate(final_number_of_columns);
        }
        self.current_grid_size.0 = final_number_of_columns;
    }
}
