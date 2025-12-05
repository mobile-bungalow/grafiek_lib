pub fn draw_grid(viewport: &egui::Rect, style: &egui::Style, painter: &egui::Painter) {
    let color = if style.visuals.dark_mode {
        egui::Color32::from_gray(30)
    } else {
        egui::Color32::from_gray(220)
    };

    let spacing = 20.0;
    let dot_radius = 1.5;

    let start_x = (viewport.left() / spacing).ceil() * spacing;
    let start_y = (viewport.top() / spacing).ceil() * spacing;

    let x_iter = 0..(viewport.width() / spacing).ceil() as usize;
    let y_iter = 0..(viewport.height() / spacing).ceil() as usize;

    let xpt_iter = x_iter.map(|i| start_x + i as f32 * spacing);
    let ypt_iter = y_iter.map(|i| start_y + i as f32 * spacing);

    // Grid
    xpt_iter
        .map(|x| ypt_iter.clone().map(move |y| (x, y)))
        .flatten()
        .for_each(|(x, y)| {
            painter.circle_filled(egui::pos2(x, y), dot_radius, color);
        });
}
