mod area;
mod box_plot;
mod bubble;
mod bullet;
mod candle;
mod cartesian;
mod donut;
mod funnel;
mod gantt;
mod heatmap;
mod histogram;
mod network;
mod parcoords;
mod radar;
mod sankey;
mod scatter;
mod stacked_area;
mod sunburst;
mod treemap;
mod violin;
mod waterfall;

use crate::data::ChartData;
use crate::engine::Chart;

/// All 20 viz chart types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChartKind {
    Area,
    BoxPlot,
    Bubble,
    Bullet,
    Candle,
    Donut,
    Funnel,
    Gantt,
    Heatmap,
    Histogram,
    Network,
    Parcoords,
    Radar,
    Sankey,
    Scatter,
    StackedArea,
    Sunburst,
    Treemap,
    Violin,
    Waterfall,
}

impl ChartKind {
    pub const ALL: [ChartKind; 20] = [
        ChartKind::Area,
        ChartKind::BoxPlot,
        ChartKind::Bubble,
        ChartKind::Bullet,
        ChartKind::Candle,
        ChartKind::Donut,
        ChartKind::Funnel,
        ChartKind::Gantt,
        ChartKind::Heatmap,
        ChartKind::Histogram,
        ChartKind::Network,
        ChartKind::Parcoords,
        ChartKind::Radar,
        ChartKind::Sankey,
        ChartKind::Scatter,
        ChartKind::StackedArea,
        ChartKind::Sunburst,
        ChartKind::Treemap,
        ChartKind::Violin,
        ChartKind::Waterfall,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            ChartKind::Area => "area_chart",
            ChartKind::BoxPlot => "box_plot",
            ChartKind::Bubble => "bubble_chart",
            ChartKind::Bullet => "bullet_chart",
            ChartKind::Candle => "candle_chart",
            ChartKind::Donut => "donut_chart",
            ChartKind::Funnel => "funnel_chart",
            ChartKind::Gantt => "gantt_chart",
            ChartKind::Heatmap => "heatmap",
            ChartKind::Histogram => "histogram",
            ChartKind::Network => "network_graph",
            ChartKind::Parcoords => "parcoords",
            ChartKind::Radar => "radar_chart",
            ChartKind::Sankey => "sankey_diagram",
            ChartKind::Scatter => "scatter_plot",
            ChartKind::StackedArea => "stacked_area_chart",
            ChartKind::Sunburst => "sunburst",
            ChartKind::Treemap => "treemap",
            ChartKind::Violin => "violin_plot",
            ChartKind::Waterfall => "waterfall_chart",
        }
    }
}

pub fn make_chart(kind: ChartKind) -> Box<dyn Chart> {
    match kind {
        ChartKind::Area => Box::new(area::AreaChart),
        ChartKind::BoxPlot => Box::new(box_plot::BoxPlot),
        ChartKind::Bubble => Box::new(bubble::BubbleChart),
        ChartKind::Bullet => Box::new(bullet::BulletChart),
        ChartKind::Candle => Box::new(candle::CandleChart),
        ChartKind::Donut => Box::new(donut::DonutChart),
        ChartKind::Funnel => Box::new(funnel::FunnelChart),
        ChartKind::Gantt => Box::new(gantt::GanttChart),
        ChartKind::Heatmap => Box::new(heatmap::Heatmap),
        ChartKind::Histogram => Box::new(histogram::Histogram),
        ChartKind::Network => Box::new(network::NetworkGraph),
        ChartKind::Parcoords => Box::new(parcoords::Parcoords),
        ChartKind::Radar => Box::new(radar::RadarChart),
        ChartKind::Sankey => Box::new(sankey::SankeyDiagram),
        ChartKind::Scatter => Box::new(scatter::ScatterPlot),
        ChartKind::StackedArea => Box::new(stacked_area::StackedArea),
        ChartKind::Sunburst => Box::new(sunburst::Sunburst),
        ChartKind::Treemap => Box::new(treemap::Treemap),
        ChartKind::Violin => Box::new(violin::ViolinPlot),
        ChartKind::Waterfall => Box::new(waterfall::Waterfall),
    }
}

// Helper shared by pie-style charts.
pub(crate) fn total(data: &ChartData) -> f64 {
    data.series
        .first()
        .map(|s| s.values.iter().sum())
        .unwrap_or(0.0)
}
