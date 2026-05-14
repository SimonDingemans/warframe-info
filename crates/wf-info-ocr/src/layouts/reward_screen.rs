use crate::layouts::text_sorting::sort_text_blocks;
use crate::pipeline::{ImageSize, ItemLayout, TextBlock, TextBounds};

#[derive(Debug, Clone)]
pub struct RewardScreenLayout {
    safe_top_ratio: f32,
    safe_bottom_ratio: f32,
    column_gap_ratio: f32,
    line_y_tolerance: f32,
}

impl Default for RewardScreenLayout {
    fn default() -> Self {
        Self {
            safe_top_ratio: 0.6,
            safe_bottom_ratio: 1.0,
            column_gap_ratio: 0.03,
            line_y_tolerance: 30.0,
        }
    }
}

impl RewardScreenLayout {
    pub fn with_safe_band(mut self, top_ratio: f32, bottom_ratio: f32) -> Self {
        self.safe_top_ratio = top_ratio;
        self.safe_bottom_ratio = bottom_ratio;
        self
    }

    pub fn with_column_gap_ratio(mut self, column_gap_ratio: f32) -> Self {
        self.column_gap_ratio = column_gap_ratio;
        self
    }
}

impl ItemLayout for RewardScreenLayout {
    type Item = String;

    fn accepts_text_bounds(&self, bounds: &TextBounds, image_size: ImageSize) -> bool {
        bounds.y_min >= image_size.height * self.safe_top_ratio
            && bounds.y_max <= image_size.height * self.safe_bottom_ratio
    }

    fn group_text_blocks(&self, blocks: &[TextBlock], image_size: ImageSize) -> Vec<Self::Item> {
        let mut sorted_blocks = blocks.to_vec();
        sorted_blocks.sort_by(|a, b| a.bounds.x_min.total_cmp(&b.bounds.x_min));

        let mut groups: Vec<Vec<TextBlock>> = Vec::new();
        let min_gap = image_size.width * self.column_gap_ratio;

        for block in sorted_blocks {
            if let Some(last_group) = groups.last_mut() {
                let group_right_edge = last_group
                    .iter()
                    .map(|existing| existing.bounds.x_max)
                    .fold(0.0, f32::max);
                if block.bounds.x_min - group_right_edge <= min_gap {
                    last_group.push(block);
                    continue;
                }
            }
            groups.push(vec![block]);
        }

        groups
            .into_iter()
            .map(|group| {
                sort_text_blocks(group, self.line_y_tolerance)
                    .into_iter()
                    .map(|block| block.text)
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .filter(|item| !item.is_empty())
            .collect()
    }
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
    fn reward_layout_uses_bottom_band_and_horizontal_clusters() {
        let layout = RewardScreenLayout::default();
        let image_size = ImageSize {
            width: 1000.0,
            height: 200.0,
        };

        assert!(!layout.accepts_text_bounds(
            &TextBounds {
                x_min: 0.0,
                y_min: 10.0,
                x_max: 100.0,
                y_max: 30.0,
            },
            image_size
        ));

        let blocks = vec![
            block("First", 100.0, 150.0, 190.0, 175.0),
            block("Part", 205.0, 150.0, 270.0, 175.0),
            block("Second", 360.0, 150.0, 470.0, 175.0),
        ];

        assert_eq!(
            layout.group_text_blocks(&blocks, image_size),
            vec!["First Part", "Second"]
        );
    }
}
