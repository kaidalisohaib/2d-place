use std::{iter, ops::DerefMut};

use bebop::{Record, SliceWrapper, SubRecord};
use tokio::sync::RwLock;

// use crate::generated::grid::owned::*;
use crate::generated::{self, grid::*};

impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Color { red, green, blue }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::new(255, 255, 255)
    }
}

pub struct State {
    pub grid: RwLock<Vec<Vec<Color>>>,
    pub delta: RwLock<Vec<Pixel>>,
    encoded_grid_data: RwLock<Vec<u8>>,
    encoded_grid: RwLock<Vec<u8>>,
    encoded_delta_data: RwLock<Vec<u8>>,
    encoded_delta: RwLock<Vec<u8>>,
    current_grid_size: (usize, usize),
}

impl State {
    pub fn new() -> Self {
        State {
            grid: RwLock::new(Vec::new()),
            delta: RwLock::new(Vec::new()),
            encoded_grid_data: RwLock::new(Vec::new()),
            encoded_grid: RwLock::new(Vec::new()),
            encoded_delta_data: RwLock::new(Vec::new()),
            encoded_delta: RwLock::new(Vec::new()),
            current_grid_size: (0, 0), // (width, height)
        }
    }

    pub fn get_grid_width(&self) -> usize {
        return self.current_grid_size.0;
    }

    pub fn get_grid_height(&self) -> usize {
        return self.current_grid_size.0;
    }

    pub async fn set_grid_size(&mut self, new_width: usize, new_height: usize) {
        self.set_grid_height(new_height).await;
        self.set_grid_width(new_width).await;
    }

    pub async fn set_grid_width(&mut self, new_width: usize) {
        // Add/Remove columns
        if new_width > self.current_grid_size.0 {
            self.add_columns(new_width - self.current_grid_size.0).await;
        } else if new_width < self.current_grid_size.0 {
            self.remove_columns(self.current_grid_size.0 - new_width)
                .await;
        }
    }

    pub async fn set_grid_height(&mut self, new_height: usize) {
        // Add/Remove rows
        if new_height > self.current_grid_size.1 {
            self.add_rows(new_height - self.current_grid_size.1).await;
        } else if new_height < self.current_grid_size.1 {
            self.remove_rows(self.current_grid_size.1 - new_height)
                .await;
        }
    }

    pub async fn add_pixel_to_delta(&self, new_pixel: Pixel) {
        self.delta.write().await.push(new_pixel);
    }

    async fn serialize_grid(&self) -> Vec<u8> {
        let grid_guard = self.grid.read().await;
        let grid = generated::grid::Grid {
            // rows: Vec::with_capacity(self.grid.len()),
            rows: grid_guard
                .iter()
                .map(|row| Row {
                    pixels: SliceWrapper::Cooked(row),
                })
                .collect(),
        };

        let mut buf: Vec<u8> = Vec::with_capacity(grid.serialized_size());
        grid.serialize(&mut buf).unwrap();

        return buf;
    }

    async fn serialize_delta(&self) -> Vec<u8> {
        let delta_guard = self.delta.read().await;
        let delta = generated::grid::DeltaGrid {
            // rows: Vec::with_capacity(self.grid.len()),
            delta: SliceWrapper::Cooked(&delta_guard),
        };

        let mut buf: Vec<u8> = Vec::with_capacity(delta.serialized_size());
        delta.serialize(&mut buf).unwrap();

        return buf;
    }

    pub async fn set_new_encoded_grid_data(&self) {
        let mut encoded_grid_guard = self.encoded_grid.write().await;
        *encoded_grid_guard = self.serialize_grid().await;
        let bebop_data = BebopData {
            protocol_version: PROTOCOL_VERSION,
            opcode: GRID_OPCODE,
            encoded_data: SliceWrapper::Raw(&encoded_grid_guard),
        };
        let mut encoded_grid_data_guard = self.encoded_grid_data.write().await;
        *encoded_grid_data_guard = Vec::with_capacity(bebop_data.serialized_size());
        bebop_data.serialize(encoded_grid_data_guard.deref_mut());
        self.delta.write().await.clear();
        self.set_new_encoded_delta_data().await;
    }

    pub async fn set_new_encoded_delta_data(&self) {
        let mut encoded_delta_guard = self.encoded_delta.write().await;

        *encoded_delta_guard = self.serialize_delta().await;
        let bebop_data = BebopData {
            protocol_version: PROTOCOL_VERSION,
            opcode: DELTA_GRID_OPCODE,
            encoded_data: SliceWrapper::Raw(&encoded_delta_guard),
        };
        let mut encoded_delta_data_guard = self.encoded_delta_data.write().await;

        *encoded_delta_data_guard = Vec::with_capacity(bebop_data.serialized_size());
        bebop_data.serialize(encoded_delta_data_guard.deref_mut());
    }

    pub async fn get_encoded_grid_data_cloned(&self) -> Vec<u8> {
        return self.encoded_grid_data.read().await.clone();
    }

    pub async fn get_encoded_delta_data_cloned(&self) -> Vec<u8> {
        return self.encoded_delta_data.read().await.clone();
    }

    pub async fn set_pixel(&self, pixel: Pixel) {
        let mut grid_guard = self.grid.write().await;

        match grid_guard.get_mut(pixel.y as usize) {
            Some(row) => match row.get_mut(pixel.x as usize) {
                Some(color) => *color = pixel.color,
                None => return,
            },
            None => return,
        }
    }
    async fn add_rows(&mut self, number_of_rows: usize) {
        let mut grid_guard = self.grid.write().await;
        grid_guard.append(
            // Repeat for each row
            &mut iter::repeat_with(|| {
                // Repeat for each column
                iter::repeat_with(|| Color::default())
                    .take(self.current_grid_size.0)
                    .collect()
            })
            .take(number_of_rows)
            .collect(),
        );

        self.current_grid_size.1 += number_of_rows;
    }

    async fn add_columns(&mut self, number_of_columns: usize) {
        let mut grid_guard = self.grid.write().await;
        for row in grid_guard.iter_mut() {
            row.append(
                &mut iter::repeat_with(|| Color::default())
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
        let mut grid_guard = self.grid.write().await;
        let final_number_of_columns = self.current_grid_size.0.saturating_sub(number_of_columns);
        for row in grid_guard.iter_mut() {
            row.truncate(final_number_of_columns);
        }
        self.current_grid_size.0 = final_number_of_columns;
    }
}
