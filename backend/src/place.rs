use std::iter;

use bebop::{Record, SliceWrapper, SubRecord};

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
    grid: Vec<Vec<Color>>,
    pub encoded_data_grid: Vec<u8>,
    encoded_grid: Vec<u8>,
    current_grid_size: (usize, usize),
}

impl State {
    pub fn new() -> Self {
        State {
            grid: Vec::new(),
            encoded_data_grid: Vec::new(),
            encoded_grid: Vec::new(),
            current_grid_size: (0, 0), // (width, height)
        }
    }

    pub fn get_grid_width(&self) -> usize {
        return self.current_grid_size.0;
    }

    pub fn get_grid_height(&self) -> usize {
        return self.current_grid_size.0;
    }

    pub fn set_grid_size(&mut self, new_width: usize, new_height: usize) {
        self.set_grid_height(new_height);
        self.set_grid_width(new_width);
    }

    pub fn set_grid_width(&mut self, new_width: usize) {
        // Add/Remove columns
        if new_width > self.current_grid_size.0 {
            self.add_columns(new_width - self.current_grid_size.0);
        } else if new_width < self.current_grid_size.0 {
            self.remove_columns(self.current_grid_size.0 - new_width);
        }
    }

    pub fn set_grid_height(&mut self, new_height: usize) {
        // Add/Remove rows
        if new_height > self.current_grid_size.1 {
            self.add_rows(new_height - self.current_grid_size.1);
        } else if new_height < self.current_grid_size.1 {
            self.remove_rows(self.current_grid_size.1 - new_height);
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let grid = generated::grid::Grid {
            // rows: Vec::with_capacity(self.grid.len()),
            rows: self
                .grid
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

    pub fn set_new_encoded_data(&mut self) {
        self.encoded_grid = self.serialize();
        let bebop_data = BebopData {
            protocol_version: PROTOCOL_VERSION,
            opcode: GRID_OPCODE,
            encoded_data: SliceWrapper::Raw(&self.encoded_grid),
        };
        self.encoded_data_grid = Vec::with_capacity(bebop_data.serialized_size());
        bebop_data.serialize(&mut self.encoded_data_grid);
    }

    pub fn get_encoded_data_cloned(&self) -> Vec<u8> {
        return self.encoded_data_grid.clone();
    }

    fn add_rows(&mut self, number_of_rows: usize) {
        self.grid.append(
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

    fn add_columns(&mut self, number_of_columns: usize) {
        for row in self.grid.iter_mut() {
            row.append(
                &mut iter::repeat_with(|| Color::default())
                    .take(number_of_columns)
                    .collect(),
            );
        }
        self.current_grid_size.0 += number_of_columns;
    }

    fn remove_rows(&mut self, number_of_rows: usize) {
        let final_number_of_rows = self.current_grid_size.1.saturating_sub(number_of_rows);
        self.grid.truncate(final_number_of_rows);
        self.current_grid_size.1 = final_number_of_rows;
    }

    fn remove_columns(&mut self, number_of_columns: usize) {
        let final_number_of_columns = self.current_grid_size.0.saturating_sub(number_of_columns);
        for row in self.grid.iter_mut() {
            row.truncate(final_number_of_columns);
        }
        self.current_grid_size.0 = final_number_of_columns;
    }
}
