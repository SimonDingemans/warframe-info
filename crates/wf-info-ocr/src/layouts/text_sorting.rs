use crate::pipeline::TextBlock;

pub(super) fn sort_text_blocks(
    mut blocks: Vec<TextBlock>,
    line_y_tolerance: f32,
) -> Vec<TextBlock> {
    blocks.sort_by(|a, b| a.bounds.center_y().total_cmp(&b.bounds.center_y()));
    let mut lines: Vec<Vec<TextBlock>> = Vec::new();

    for block in blocks {
        if let Some(last_line) = lines.last_mut() {
            if belongs_to_line(&block, last_line, line_y_tolerance) {
                last_line.push(block);
                continue;
            }
        }
        lines.push(vec![block]);
    }

    lines.sort_by(|a, b| average_line_y(a).total_cmp(&average_line_y(b)));
    lines
        .into_iter()
        .flat_map(|mut line| {
            line.sort_by(|a, b| a.bounds.x_min.total_cmp(&b.bounds.x_min));
            line
        })
        .collect()
}

fn average_line_y(line: &[TextBlock]) -> f32 {
    line.iter()
        .map(|block| block.bounds.center_y())
        .sum::<f32>()
        / line.len() as f32
}

fn belongs_to_line(block: &TextBlock, line: &[TextBlock], line_y_tolerance: f32) -> bool {
    let center_delta = (block.bounds.center_y() - average_line_y(line)).abs();
    if center_delta > line_y_tolerance {
        return false;
    }

    let overlap_ratio = line
        .iter()
        .map(|line_block| vertical_overlap_ratio(&block.bounds, &line_block.bounds))
        .fold(0.0, f32::max);
    if overlap_ratio >= 0.25 {
        return true;
    }

    center_delta <= average_line_height(line).max(block.bounds.height()) * 0.5
}

fn average_line_height(line: &[TextBlock]) -> f32 {
    line.iter().map(|block| block.bounds.height()).sum::<f32>() / line.len() as f32
}

fn vertical_overlap_ratio(a: &crate::pipeline::TextBounds, b: &crate::pipeline::TextBounds) -> f32 {
    let overlap = a.y_max.min(b.y_max) - a.y_min.max(b.y_min);
    if overlap <= 0.0 {
        return 0.0;
    }

    overlap / a.height().min(b.height()).max(f32::EPSILON)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::TextBounds;

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
    fn sorter_keeps_stacked_centered_words_in_reading_order() {
        let blocks = vec![
            block("Concentration", 100.0, 126.0, 260.0, 146.0),
            block("Arcane", 145.0, 100.0, 220.0, 120.0),
        ];

        let sorted: Vec<String> = sort_text_blocks(blocks, 30.0)
            .into_iter()
            .map(|block| block.text)
            .collect();

        assert_eq!(sorted, vec!["Arcane", "Concentration"]);
    }

    #[test]
    fn sorter_keeps_same_line_words_left_to_right() {
        let blocks = vec![
            block("Prime", 190.0, 102.0, 260.0, 122.0),
            block("Acceltra", 100.0, 100.0, 180.0, 120.0),
        ];

        let sorted: Vec<String> = sort_text_blocks(blocks, 30.0)
            .into_iter()
            .map(|block| block.text)
            .collect();

        assert_eq!(sorted, vec!["Acceltra", "Prime"]);
    }
}
