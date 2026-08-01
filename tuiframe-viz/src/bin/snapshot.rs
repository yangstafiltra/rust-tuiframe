use std::fs;

use ratatui::backend::TestBackend;
use ratatui::Terminal;
use tuiframe_viz::{ChartData, ChartKind, make_chart};

fn main() {
    let out_dir = std::env::args().nth(1).unwrap_or_else(|| "/tmp/viz_snap".into());
    fs::create_dir_all(&out_dir).expect("create out dir");

    for kind in ChartKind::ALL {
        let mut chart = make_chart(kind);
        let mut terminal = Terminal::new(TestBackend::new(96, 30)).expect("backend");
        terminal.clear().ok();
        let presets = chart.presets();
        let first = presets
            .first()
            .cloned()
            .unwrap_or_else(|| ChartData::single(vec![0.0]));
        terminal
            .draw(|f| {
                let area = f.area();
                let inner = ratatui::layout::Rect::new(area.x, area.y, area.width, area.height);
                chart.render(f.buffer_mut(), inner, &first);
            })
            .ok();
        let rendered = terminal.backend().buffer();
        let mut out = String::new();
        for y in 0..rendered.area.height {
            for x in 0..rendered.area.width {
                let cell = rendered.cell((x, y)).unwrap();
                out.push_str(cell.symbol());
            }
            out.push('\n');
        }
        let name = kind.name();
        fs::write(format!("{}/{}.txt", out_dir, name), &out).expect("write snapshot");
        println!("{} rendered {}x{}", name, rendered.area.width, rendered.area.height);
    }
    println!("snapshots -> {}", out_dir);
}
