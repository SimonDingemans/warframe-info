use crate::layouts::text_sorting::sort_text_blocks;
use crate::pipeline::{ImageSize, ItemLayout, TextBlock, TextBounds};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct InventoryGridLayout {
    columns: usize,
    safe_top_ratio: f32,
    safe_bottom_ratio: f32,
    line_y_tolerance: f32,
}

impl Default for InventoryGridLayout {
    fn default() -> Self {
        Self {
            columns: 6,
            safe_top_ratio: 0.12,
            safe_bottom_ratio: 0.85,
            line_y_tolerance: 30.0,
        }
    }
}

impl InventoryGridLayout {
    pub fn new(columns: usize) -> Self {
        Self {
            columns,
            ..Self::default()
        }
    }

    pub fn with_safe_band(mut self, top_ratio: f32, bottom_ratio: f32) -> Self {
        self.safe_top_ratio = top_ratio;
        self.safe_bottom_ratio = bottom_ratio;
        self
    }

    pub fn with_line_y_tolerance(mut self, line_y_tolerance: f32) -> Self {
        self.line_y_tolerance = line_y_tolerance;
        self
    }
}

impl ItemLayout for InventoryGridLayout {
    type Item = String;

    fn should_recover_stacked_text_blocks(&self) -> bool {
        true
    }

    fn accepts_text_bounds(&self, bounds: &TextBounds, image_size: ImageSize) -> bool {
        bounds.y_min >= image_size.height * self.safe_top_ratio
            && bounds.y_max <= image_size.height * self.safe_bottom_ratio
    }

    fn group_text_blocks(&self, blocks: &[TextBlock], image_size: ImageSize) -> Vec<Self::Item> {
        let positioned_blocks = assign_inventory_grid_positions(blocks, image_size, self.columns);
        let mut grid: BTreeMap<(usize, usize), Vec<PositionedTextBlock>> = BTreeMap::new();

        for block in positioned_blocks {
            grid.entry((block.row, block.col)).or_default().push(block);
        }

        grid.into_values()
            .map(|cell_blocks| {
                sort_cell_blocks(cell_blocks, self.line_y_tolerance)
                    .into_iter()
                    .map(|block| block.text.text)
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .filter(|item| !item.is_empty())
            .collect()
    }
}

#[derive(Debug, Clone)]
struct PositionedTextBlock {
    text: TextBlock,
    row: usize,
    col: usize,
}

fn assign_inventory_grid_positions(
    blocks: &[TextBlock],
    image_size: ImageSize,
    columns: usize,
) -> Vec<PositionedTextBlock> {
    let columns = columns.max(1);
    let column_width = image_size.width / columns as f32;
    let row_gap_threshold = image_size.height * 0.12;
    let mut sorted_y: Vec<f32> = blocks.iter().map(|block| block.bounds.center_y()).collect();
    sorted_y.sort_by(|a, b| a.total_cmp(b));

    let mut row_centers: Vec<Vec<f32>> = Vec::new();
    for center_y in sorted_y {
        if let Some(last_row) = row_centers.last_mut() {
            let last_y = *last_row.last().expect("row cannot be empty");
            if center_y - last_y > row_gap_threshold {
                row_centers.push(vec![center_y]);
            } else {
                last_row.push(center_y);
            }
        } else {
            row_centers.push(vec![center_y]);
        }
    }

    let row_anchors: Vec<f32> = row_centers
        .iter()
        .map(|row| row.iter().sum::<f32>() / row.len() as f32)
        .collect();

    blocks
        .iter()
        .filter_map(|block| {
            let col = (block.bounds.center_x() / column_width) as usize;
            let (row, _) = row_anchors.iter().enumerate().min_by(|(_, a), (_, b)| {
                (block.bounds.center_y() - *a)
                    .abs()
                    .total_cmp(&(block.bounds.center_y() - *b).abs())
            })?;

            Some(PositionedTextBlock {
                text: block.clone(),
                row,
                col: col.clamp(0, columns.saturating_sub(1)),
            })
        })
        .collect()
}

fn sort_cell_blocks(
    blocks: Vec<PositionedTextBlock>,
    line_y_tolerance: f32,
) -> Vec<PositionedTextBlock> {
    let sorted = sort_text_blocks(
        blocks.into_iter().map(|block| block.text).collect(),
        line_y_tolerance,
    );

    sorted
        .into_iter()
        .map(|text| PositionedTextBlock {
            text,
            row: 0,
            col: 0,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(text: &str, x_min: f32, y_min: f32, x_max: f32, y_max: f32) -> TextBlock {
        TextBlock {
            text: text.to_string(),
            score: 1.0,
            bounds: TextBounds {
                x_min,
                y_min,
                x_max,
                y_max,
            },
        }
    }

    #[test]
    fn inventory_layout_groups_text_by_grid_cell() {
        let layout = InventoryGridLayout::new(2).with_safe_band(0.0, 1.0);
        let image_size = ImageSize {
            width: 1000.0,
            height: 1000.0,
        };
        let blocks = vec![
            block("Left", 100.0, 100.0, 180.0, 130.0),
            block("Cell", 190.0, 100.0, 260.0, 130.0),
            block("Right", 650.0, 100.0, 760.0, 130.0),
            block("Lower", 100.0, 400.0, 210.0, 430.0),
        ];

        assert_eq!(
            layout.group_text_blocks(&blocks, image_size),
            vec!["Left Cell", "Right", "Lower"]
        );
    }
}
