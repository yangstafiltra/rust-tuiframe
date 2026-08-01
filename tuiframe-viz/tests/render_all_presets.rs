use ratatui::backend::TestBackend;
use ratatui::Terminal;
use ratatui::layout::Rect;
use tuiframe_viz::data::interpolate;
use tuiframe_viz::{ChartKind, make_chart};

#[test]
fn render_all_presets_no_panic() {
    for kind in ChartKind::ALL {
        let mut chart = make_chart(kind);
        let presets = chart.presets();
        for (i, data) in presets.iter().enumerate() {
            let mut terminal = Terminal::new(TestBackend::new(96, 30)).expect("backend");
            let r = terminal.draw(|f| {
                let area = f.area();
                chart.render(f.buffer_mut(), area, data);
            });
            assert!(r.is_ok(), "render failed: {:?} preset {}", kind.name(), i + 1);
        }
        println!("{}: {} presets OK", kind.name(), presets.len());
    }
}

#[test]
fn render_all_interpolated_states_no_panic() {
    for kind in ChartKind::ALL {
        let mut chart = make_chart(kind);
        let presets = chart.presets();
        for (i, data) in presets.iter().enumerate() {
            let mut terminal = Terminal::new(TestBackend::new(96, 30)).expect("backend");
            let mid = interpolate(&presets[0], data, 0.4, |t| t);
            let r = terminal.draw(|f| {
                let area = f.area();
                chart.render(f.buffer_mut(), area, &mid);
            });
            assert!(r.is_ok(), "mid-state render failed: {} preset {}", kind.name(), i + 1);
        }
        println!("{}: {} mid-states OK", kind.name(), presets.len());
    }
}

#[test]
fn render_chain_interpolated_states_no_panic() {
    for kind in ChartKind::ALL {
        let mut chart = make_chart(kind);
        let presets = chart.presets();
        let mut state = presets[0].clone();
        for (i, data) in presets.iter().enumerate() {
            for t in [0.0, 0.15, 0.5, 0.85, 1.0] {
                let mid = interpolate(&state, data, t, |x| x);
                let mut terminal = Terminal::new(TestBackend::new(96, 30)).expect("backend");
                let r = terminal.draw(|f| {
                    let area = f.area();
                    chart.render(f.buffer_mut(), area, &mid);
                });
                assert!(r.is_ok(), "chain render failed: {} preset {} t {}", kind.name(), i + 1, t);
            }
            state = interpolate(&state, data, 0.5, |x| x);
        }
        println!("{}: chain OK", kind.name());
    }
}

#[test]
fn render_bidirectional_interpolation_no_panic() {
    // Switching between presets of *different* label counts (e.g. parcoords
    // 4-dim vs 5-dim) can produce interpolated series longer than the target
    // labels. Rendering every pairwise mid-state must not panic.
    for kind in ChartKind::ALL {
        let mut chart = make_chart(kind);
        let presets = chart.presets();
        for (i, a) in presets.iter().enumerate() {
            for (j, b) in presets.iter().enumerate() {
                for t in [0.0, 0.3, 0.5, 0.7, 1.0] {
                    let mid = interpolate(a, b, t, |x| x);
                    let mut terminal = Terminal::new(TestBackend::new(96, 30)).expect("backend");
                    let r = terminal.draw(|f| {
                        let area = f.area();
                        chart.render(f.buffer_mut(), area, &mid);
                    });
                    assert!(r.is_ok(), "pairwise mid render failed: {} {}->{} t {}", kind.name(), i + 1, j + 1, t);
                }
            }
        }
        println!("{}: {}x{} pairwise mid-states OK", kind.name(), presets.len(), presets.len());
    }
}

#[test]
fn heatmap_resolution_affects_subdivision() {
    use ratatui::style::Color;
    let mut chart = make_chart(ChartKind::Heatmap);
    let data = chart.presets()[0].clone();
    // Small terminal
    let mut small = Terminal::new(TestBackend::new(30, 10)).expect("small");
    let sa = Rect::new(0, 0, 30, 10);
    small.draw(|f| chart.render(f.buffer_mut(), sa, &data)).ok();
    // Large terminal
    let mut big = Terminal::new(TestBackend::new(200, 60)).expect("big");
    let ba = Rect::new(0, 0, 200, 60);
    big.draw(|f| chart.render(f.buffer_mut(), ba, &data)).ok();
    // Count distinct colors per terminal — bigger should have far more unique shades.
    let count_unique = |t: &Terminal<TestBackend>| {
        let buf = t.backend().buffer();
        let mut colors = std::collections::HashSet::new();
        for x in 0..buf.area.width {
            for y in 0..buf.area.height {
                if let Some(c) = buf.cell((x, y)) {
                    if let Color::Rgb(r, g, b) = c.fg {
                        colors.insert((r, g, b));
                    }
                }
            }
        }
        colors.len()
    };
    let s = count_unique(&small);
    let b = count_unique(&big);
    assert!(b > s, "big terminal {} colors should exceed small {} colors", b, s);
    println!("heatmap unique colors: small={} big={}", s, b);
}
