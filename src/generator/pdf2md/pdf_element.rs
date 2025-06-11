#[derive(Clone)]
pub enum PdfElement {
    Text(PdfText),
    Table(PdfTable),
}
pub enum PdfUnit {
    Text(PdfText),
    Line(PdfLine),
}

#[derive(Clone, Debug)]
pub struct PdfLine {
    pub from: (f32, f32),
    pub to: (f32, f32),
}

#[derive(Default, Clone, Debug)]
pub struct PdfText {
    pub text: String,
    pub italic: bool,
    pub font_name: Option<String>,
    pub font_size: Option<f32>,
    pub x: f32,
    pub y: f32,
    pub underlined: bool,
    pub color: Option<String>,
}

#[derive(Default, Clone, Debug)]
struct TableBoundary {
    minx: f32,
    maxx: f32,
    miny: f32,
    maxy: f32,
    elements: Vec<PdfText>,
}
#[derive(Default, Clone, Debug)]
pub struct PdfTable {
    boundaries: Vec<TableBoundary>,
    y: f32, // just the center of it
    x: f32, // just the center of it
}

impl PdfElement {
    pub fn get_y(&self) -> f32 {
        match self {
            PdfElement::Text(pdf_text) => pdf_text.y,
            PdfElement::Table(pdf_table) => pdf_table.y,
        }
    }
    pub fn get_x(&self) -> f32 {
        match self {
            PdfElement::Text(pdf_text) => pdf_text.x,
            PdfElement::Table(pdf_table) => pdf_table.x,
        }
    }
}

fn distance(pt1: (f32, f32), pt2: (f32, f32)) -> f32 {
    ((pt1.0 - pt2.0).powi(2) + (pt1.1 - pt2.1).powi(2)).sqrt()
}

fn cluster_points(points: &[(f32, f32)], radius: f32) -> Vec<(f32, f32)> {
    let n = points.len();

    // Initialize Union-Find structure
    let mut parents: Vec<usize> = (0..n).collect();
    let mut sizes: Vec<usize> = vec![1; n];

    // Find function with path compression
    fn find(parents: &mut Vec<usize>, x: usize) -> usize {
        if parents[x] != x {
            parents[x] = find(parents, parents[x]);
        }
        parents[x]
    }

    // Union function with rank optimization
    fn union(parents: &mut Vec<usize>, sizes: &mut Vec<usize>, x: usize, y: usize) {
        let root_x = find(parents, x);
        let root_y = find(parents, y);

        if root_x == root_y {
            return;
        }

        if sizes[root_x] < sizes[root_y] {
            parents[root_x] = root_y;
            sizes[root_y] += sizes[root_x];
        } else {
            parents[root_y] = root_x;
            sizes[root_x] += sizes[root_y];
        }
    }

    // Connect points within radius
    for i in 0..n {
        let (x1, y1) = points[i];
        for j in (i + 1)..n {
            let (x2, y2) = points[j];
            let dx = x2 - x1;
            let dy = y2 - y1;
            if (dx * dx + dy * dy).sqrt() <= radius {
                union(&mut parents, &mut sizes, i, j);
            }
        }
    }

    // Create map of root -> points in cluster
    let mut clusters: std::collections::HashMap<usize, Vec<(f32, f32)>> =
        std::collections::HashMap::new();
    for i in 0..n {
        let root = find(&mut parents, i);
        clusters
            .entry(root)
            .or_insert_with(Vec::new)
            .push(points[i]);
    }

    // Calculate centroids
    let mut centroids = Vec::new();
    for points in clusters.values() {
        let len = points.len() as f32;
        let sum = points
            .iter()
            .fold((0.0, 0.0), |acc, &(x, y)| (acc.0 + x, acc.1 + y));
        centroids.push((sum.0 / len, sum.1 / len));
    }

    centroids
}

fn quantize(value: f32, epsilon: f32) -> i32 {
    (value / epsilon).round() as i32
}

fn intersections_to_table(mut intersections: Vec<(f32, f32)>) -> Option<PdfTable> {
    // sort top-left to bottom-right
    let epsilon = 3.0;
    intersections.sort_by(|&(ax, ay), &(bx, by)| {
        let ayq = quantize(ay, epsilon);
        let byq = quantize(by, epsilon);

        match ayq.cmp(&byq) {
            std::cmp::Ordering::Equal => {
                let axq = quantize(ax, epsilon);
                let bxq = quantize(bx, epsilon);
                axq.cmp(&bxq)
            }
            ord => ord,
        }
    });

    let mut rows: Vec<Vec<(f32, f32)>> = Vec::new();
    let mut current_row: Vec<(f32, f32)> = Vec::new();

    // into a matrix
    for pt in intersections {
        if current_row.is_empty() {
            current_row.push(pt);
        } else {
            let last_y = current_row[0].1;
            if (pt.1 - last_y).abs() <= epsilon {
                current_row.push(pt);
            } else {
                rows.push(current_row);
                current_row = vec![pt];
            }
        }
    }
    if !current_row.is_empty() {
        rows.push(current_row);
    }

    // sort the rows left to right
    for row in rows.iter_mut() {
        row.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    }

    let mut boundaries = Vec::new();
    for y in 0..rows.len().saturating_sub(1) {
        let row = &rows[y];
        let next_row = &rows[y + 1];

        let cols = row.len().min(next_row.len());
        for x in 0..cols - 1 {
            let top_left = row[x];
            let bottom_right = next_row[x + 1];

            boundaries.push(TableBoundary {
                minx: top_left.0,
                maxx: bottom_right.0,
                miny: top_left.1,
                maxy: bottom_right.1,
                elements: vec![],
            });
        }
    }

    // Estimate table center
    let top_left = rows
        .first()
        .and_then(|r| r.first())
        .copied()
        .unwrap_or((0.0, 0.0));
    let bottom_right = rows
        .last()
        .and_then(|r| r.last())
        .copied()
        .unwrap_or((0.0, 0.0));

    let x = (top_left.0 + bottom_right.0) / 2.0;
    let y = (top_left.1 + bottom_right.1) / 2.0;

    if boundaries.len() <= 1 {
        return None;
    }
    Some(PdfTable { boundaries, x, y })
}

fn lines_to_intersections(lines: Vec<PdfLine>) -> Vec<(f32, f32)> {
    // First, separate horizontal and vertical lines
    let mut horizontal_lines: Vec<&PdfLine> = Vec::new();
    let mut vertical_lines: Vec<&PdfLine> = Vec::new();
    for line in &lines {
        let (x1, y1) = line.from;
        let (x2, y2) = line.to;

        let epsilon = 2.0;

        if (y1 - y2).abs() < epsilon {
            horizontal_lines.push(line);
        } else if (x1 - x2).abs() < epsilon {
            vertical_lines.push(line);
        }
    }
    // If we don't have both horizontal and vertical lines, we can't form a table
    if horizontal_lines.is_empty() || vertical_lines.is_empty() {
        return Vec::new();
    }

    // Find intersections between horizontal and vertical lines
    let mut intersections: Vec<(f32, f32)> = Vec::new();
    let epsilon = 10.0;

    for h_line in &horizontal_lines {
        let h_y = h_line.from.1; // Y-coordinate is the same for horizontal lines
        let h_min_x = h_line.from.0.min(h_line.to.0);
        let h_max_x = h_line.from.0.max(h_line.to.0);

        for v_line in &vertical_lines {
            let v_x = v_line.from.0; // X-coordinate is the same for vertical lines
            let v_min_y = v_line.from.1.min(v_line.to.1);
            let v_max_y = v_line.from.1.max(v_line.to.1);

            // Check if lines intersect
            if (v_x - h_min_x).abs() <= epsilon
                || (v_x - h_max_x).abs() <= epsilon
                || (v_x >= h_min_x && v_x <= h_max_x)
            {
                if (h_y - v_min_y).abs() <= epsilon
                    || (h_y - v_max_y).abs() <= epsilon
                    || (h_y >= v_min_y && h_y <= v_max_y)
                {
                    // Calculate potential intersection point
                    let new_point = (v_x, h_y);

                    intersections.push(new_point);
                }
            }
        }
    }

    cluster_points(&intersections, epsilon)
}

fn cluster_lines(lines: Vec<PdfLine>) -> Vec<Vec<PdfLine>> {
    let epsilon = 10.0;

    fn fixed_lines_close(line: &PdfLine, other: &PdfLine, epsilon: f32) -> bool {
        distance(line.from, other.to) < epsilon
            || distance(line.from, other.from) < epsilon
            || distance(line.to, other.to) < epsilon
            || distance(line.to, other.from) < epsilon
    }

    // Create a disjoint-set (union-find) data structure for tracking clusters
    let mut parents: Vec<usize> = (0..lines.len()).collect();
    let mut sizes: Vec<usize> = vec![1; lines.len()];

    // Find function with path compression
    fn find(parents: &mut Vec<usize>, x: usize) -> usize {
        if parents[x] != x {
            parents[x] = find(parents, parents[x]);
        }
        parents[x]
    }

    // Union function with rank (size) heuristic
    fn union(parents: &mut Vec<usize>, sizes: &mut Vec<usize>, x: usize, y: usize) {
        let root_x = find(parents, x);
        let root_y = find(parents, y);

        if root_x == root_y {
            return;
        }

        // Attach smaller tree under root of larger tree
        if sizes[root_x] < sizes[root_y] {
            parents[root_x] = root_y;
            sizes[root_y] += sizes[root_x];
        } else {
            parents[root_y] = root_x;
            sizes[root_x] += sizes[root_y];
        }
    }

    // Build connections between lines using union-find
    for i in 0..lines.len() {
        for j in (i + 1)..lines.len() {
            if fixed_lines_close(&lines[i], &lines[j], epsilon) {
                union(&mut parents, &mut sizes, i, j);
            }
        }
    }

    // Collect lines into their respective clusters
    let mut cluster_map: std::collections::HashMap<usize, Vec<PdfLine>> =
        std::collections::HashMap::new();

    for i in 0..lines.len() {
        let root = find(&mut parents, i);
        cluster_map
            .entry(root)
            .or_insert_with(Vec::new)
            .push(lines[i].clone());
    }

    // Convert the hashmap into a Vec<Vec<PdfLine>>
    cluster_map.into_values().collect()
}

fn deduplicate_lines(lines: Vec<PdfLine>) -> Vec<PdfLine> {
    let mut unique: Vec<PdfLine> = Vec::new();
    let epsilon = 5.0;

    for line in lines.iter() {
        let mut found = false;
        for u in unique.iter() {
            if distance(line.from, u.from) < epsilon && distance(u.to, line.to) < epsilon {
                found = true;
                break;
            }
        }
        if !found {
            unique.push(line.clone());
        }
    }

    unique
}

impl PdfTable {
    pub fn from_lines(lines: Vec<PdfLine>) -> Vec<PdfTable> {
        // deduplicate_lines
        let lines = deduplicate_lines(lines);
        // seperate them into vertical and horizontal lines
        let line_clusters = cluster_lines(lines);

        let mut pdf_tables = Vec::new();
        for lines in line_clusters {
            let intersections = lines_to_intersections(lines);
            if intersections.is_empty() {
                continue;
            }

            if let Some(pdf_table) = intersections_to_table(intersections) {
                pdf_tables.push(pdf_table);
            }
        }

        pdf_tables
    }

    pub fn assign(&mut self, element: &PdfText) -> bool {
        for boundary in self.boundaries.iter_mut() {
            if boundary.assign(element) {
                return true;
            }
        }
        false
    }

    pub fn get_sorted_elements(&mut self) -> Vec<Vec<Vec<PdfText>>> {
        self.boundaries.sort_by(|a, b| {
            b.miny.partial_cmp(&a.miny).unwrap().then(
                a.minx
                    .partial_cmp(&b.minx)
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
        });
        const Y_THRESHOLD: f32 = 1.0;

        let mut rows: Vec<Vec<TableBoundary>> = Vec::new();
        for boundary in self.boundaries.iter() {
            let mut matched = false;
            for row in &mut rows {
                // Compare with the *row's reference y*, e.g. the first cell's miny
                if (row[0].miny - boundary.miny).abs() < Y_THRESHOLD {
                    row.push(boundary.clone());
                    matched = true;
                    break;
                }
            }
            if !matched {
                rows.push(vec![boundary.clone()]);
            }
        }

        for row in &mut rows {
            row.sort_by(|a, b| {
                a.minx
                    .partial_cmp(&b.minx)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        let result: Vec<Vec<Vec<PdfText>>> = rows
            .iter_mut()
            .map(|row| {
                row.iter_mut()
                    .map(|cell| cell.clone().get_sorted_elements())
                    .collect()
            })
            .collect();

        result
    }
}

impl TableBoundary {
    pub fn assign(&mut self, element: &PdfText) -> bool {
        if element.x > self.minx
            && element.x < self.maxx
            && element.y < self.maxy
            && element.y > self.miny
        {
            self.elements.push(element.clone());
            return true;
        }

        false
    }

    pub fn get_sorted_elements(self) -> Vec<PdfText> {
        sort_transform_row(self.elements)
    }
}

pub fn units_to_elements(units: Vec<PdfUnit>) -> Vec<PdfElement> {
    let (texts, lines): (Vec<_>, Vec<_>) =
        units
            .into_iter()
            .fold((vec![], vec![]), |(mut texts, mut lines), unit| {
                match unit {
                    PdfUnit::Text(t) => texts.push(t),
                    PdfUnit::Line(l) => lines.push(l),
                }
                (texts, lines)
            });
    let mut tables = PdfTable::from_lines(lines);
    // assign to the tables
    let texts = if !tables.is_empty() {
        let mut new_texts = Vec::new();
        let mut assigned = false;
        for text in texts.iter() {
            for table in tables.iter_mut() {
                if table.assign(text) {
                    assigned = true;
                }
            }
            if !assigned {
                new_texts.push(text.clone());
            }
        }
        new_texts
    } else {
        texts
    };

    let mut elements: Vec<PdfElement> = Vec::new();
    let texts: Vec<PdfElement> = texts.into_iter().map(PdfElement::Text).collect();
    let tables: Vec<PdfElement> = tables.into_iter().map(PdfElement::Table).collect();
    elements.extend(texts);
    elements.extend(tables);
    elements
}

pub fn sort_transform_row(mut elements: Vec<PdfText>) -> Vec<PdfText> {
    elements.sort_by(|a, b| match b.y.partial_cmp(&a.y) {
        Some(std::cmp::Ordering::Equal) => {
            a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal)
        }
        Some(ordering) => ordering,
        None => std::cmp::Ordering::Equal,
    });

    elements
}

pub fn sort_transform_elements(elements: &mut Vec<PdfElement>) {
    elements.sort_by(|a, b| match b.get_y().partial_cmp(&a.get_y()) {
        Some(std::cmp::Ordering::Equal) => a
            .get_x()
            .partial_cmp(&b.get_x())
            .unwrap_or(std::cmp::Ordering::Equal),
        Some(ordering) => ordering,
        None => std::cmp::Ordering::Equal,
    });
}

pub fn elements_into_matrix(mut elements: Vec<PdfElement>) -> Vec<Vec<PdfElement>> {
    if elements.is_empty() {
        return Vec::new();
    }

    elements.sort_by(|a, b| {
        b.get_y()
            .partial_cmp(&a.get_y())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Group into rows
    let mut result: Vec<Vec<PdfElement>> = Vec::new();
    let mut current_row: Vec<PdfElement> = Vec::new();
    let mut current_y = elements[0].get_y();
    for element in elements {
        if (current_y - element.get_y()).abs() > 1.0 {
            if !current_row.is_empty() {
                result.push(current_row);
                current_row = Vec::new();
            }
            current_y = element.get_y();
        }
        current_row.push(element);
    }
    if !current_row.is_empty() {
        result.push(current_row);
    }

    // Compute row heights
    let row_heights: Vec<f32> = result
        .iter()
        .map(|row| {
            row.iter()
                .map(|text| text.get_y())
                .fold(f32::NEG_INFINITY, f32::max)
        })
        .collect();

    // Compute gaps
    let mut gaps = Vec::new();
    for i in 0..row_heights.len() - 1 {
        let gap = row_heights[i] - row_heights[i + 1];
        gaps.push(gap);
    }

    if gaps.is_empty() {
        return result;
    }

    // Insert spacer *after* row[i+1] if gap[i+1] > gap[i] * 1.2
    let mut final_result = Vec::new();
    for i in 0..result.len() {
        final_result.push(result[i].clone());
        if i == 0 && gaps[0] > 20.0 {
            final_result.push(Vec::new());
        }

        if i > 0 && i < gaps.len() {
            let prev_gap = gaps[i - 1];
            let curr_gap = gaps[i];

            if curr_gap > prev_gap * 1.3 {
                final_result.push(Vec::new()); // spacer
            }
        }
    }

    final_result
}