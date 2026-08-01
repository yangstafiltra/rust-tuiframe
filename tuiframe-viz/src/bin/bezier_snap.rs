use ratatui::backend::TestBackend;
use ratatui::Terminal;
use tuiframe_viz::bezier_editor::BezierEditor;

fn main() {
    let w: u16 = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(100);
    let h: u16 = std::env::args().nth(2).and_then(|a| a.parse().ok()).unwrap_or(32);
    let preset: usize = std::env::args().nth(3).and_then(|a| a.parse().ok()).unwrap_or(3);
    let out = std::env::args().nth(4).unwrap_or_else(|| "/tmp/bezier_snap.txt".into());
    let mut terminal = Terminal::new(TestBackend::new(w, h)).expect("backend");
    terminal.clear().ok();
    let mut editor = BezierEditor::new();
    editor.apply_preset(preset);
    editor.tick(0.4);
    terminal
        .draw(|f| {
            let area = f.area();
            editor.render(f.buffer_mut(), area);
        })
        .ok();
    let rendered = terminal.backend().buffer();
    let mut s = String::new();
    for y in 0..rendered.area.height {
        for x in 0..rendered.area.width {
            s.push_str(rendered.cell((x, y)).unwrap().symbol());
        }
        s.push('\n');
    }
    std::fs::write(&out, &s).expect("write");
    println!("bezier editor {w}x{h} -> {out}");
}
