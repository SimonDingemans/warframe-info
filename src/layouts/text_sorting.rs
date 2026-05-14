use crate::pipeline::TextBlock;

pub(super) fn sort_text_blocks(
    mut blocks: Vec<TextBlock>,
    line_y_tolerance: f32,
) -> Vec<TextBlock> {
    blocks.sort_by(|a, b| a.bounds.center_y().total_cmp(&b.bounds.center_y()));
    let mut lines: Vec<Vec<TextBlock>> = Vec::new();

    for block in blocks {
        if let Some(last_line) = lines.last_mut() {
            if (block.bounds.center_y() - average_line_y(last_line)).abs() <= line_y_tolerance {
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
