//! Plot ease_noise.png and fractal_noise.png for documentation.

use fundsp::hacker32::*;
use plotters::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Plot fractal_noise.
    let root = BitMapBackend::new("fractal_noise.png", (1280, 640)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("fractal_noise, 8 octaves", ("sans-serif", 40).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(-2f32..2f32, -1f32..1f32)?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(
            (-200..=200)
                .map(|x| x as f32 / 100.0)
                .map(|x| (x, fractal_noise(5, 8, 0.3, x))),
            RGBColor(0, 64, 192).stroke_width(2),
        ))?
        .label("roughness 0.3")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                RGBColor(0, 64, 192).stroke_width(2),
            )
        });

    chart
        .draw_series(LineSeries::new(
            (-200..=200)
                .map(|x| x as f32 / 100.0)
                .map(|x| (x, fractal_noise(0, 8, 0.5, x))),
            RGBColor(192, 0, 0).stroke_width(2),
        ))?
        .label("roughness 0.5")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                RGBColor(192, 0, 0).stroke_width(2),
            )
        });

    chart
        .draw_series(LineSeries::new(
            (-200..=200)
                .map(|x| x as f32 / 100.0)
                .map(|x| (x, fractal_noise(2, 8, 0.7, x))),
            RGBColor(0, 192, 0).stroke_width(2),
        ))?
        .label("roughness 0.7")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                RGBColor(0, 192, 0).stroke_width(2),
            )
        });

    chart
        .draw_series(LineSeries::new(
            (-200..=200)
                .map(|x| x as f32 / 100.0)
                .map(|x| (x, fractal_noise(108, 8, 0.9, x))),
            RGBColor(96, 96, 96).stroke_width(2),
        ))?
        .label("roughness 0.9")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                RGBColor(96, 96, 96).stroke_width(2),
            )
        });

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    // Plot ease_noise.
    let root = BitMapBackend::new("ease_noise.png", (1280, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption(
            "ease_noise easing functions",
            ("sans-serif", 40).into_font(),
        )
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(-2f32..2f32, -1f32..1f32)?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(
            (-200..=200)
                .map(|x| x as f32 / 100.0)
                .map(|x| (x, ease_noise(smooth9, 13, x))),
            RGBColor(0, 64, 192).stroke_width(2),
        ))?
        .label("smooth9")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                RGBColor(0, 64, 192).stroke_width(2),
            )
        });

    chart
        .draw_series(LineSeries::new(
            (-200..=200)
                .map(|x| x as f32 / 100.0)
                .map(|x| (x, ease_noise((uparc, downarc), 4, x))),
            RGBColor(192, 0, 0).stroke_width(2),
        ))?
        .label("(uparc, downarc)")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                RGBColor(192, 0, 0).stroke_width(2),
            )
        });

    chart
        .draw_series(LineSeries::new(
            (-200..=200)
                .map(|x| x as f32 / 100.0)
                .map(|x| (x, ease_noise(smooth3, 12, x))),
            RGBColor(0, 192, 0).stroke_width(2),
        ))?
        .label("smooth3")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                RGBColor(0, 192, 0).stroke_width(2),
            )
        });

    chart
        .draw_series(LineSeries::new(
            (-200..=200)
                .map(|x| x as f32 / 100.0)
                .map(|x| (x, ease_noise(id, 100, x))),
            RGBColor(96, 96, 96).stroke_width(2),
        ))?
        .label("id")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                RGBColor(96, 96, 96).stroke_width(2),
            )
        });

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    // Plot fractal_ease_noise.
    let root = BitMapBackend::new("fractal_ease_noise.png", (1280, 640)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption(
            "fractal_ease_noise, roughness 0.7, 4 octaves",
            ("sans-serif", 40).into_font(),
        )
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(-2f32..2f32, -1f32..1f32)?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(
            (-200..=200)
                .map(|x| x as f32 / 100.0)
                .map(|x| (x, fractal_ease_noise(smooth9, 5, 4, 0.7, x))),
            RGBColor(0, 64, 192).stroke_width(2),
        ))?
        .label("smooth9")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                RGBColor(0, 64, 192).stroke_width(2),
            )
        });

    chart
        .draw_series(LineSeries::new(
            (-200..=200)
                .map(|x| x as f32 / 100.0)
                .map(|x| (x, fractal_ease_noise((uparc, downarc), 0, 4, 0.7, x))),
            RGBColor(192, 0, 0).stroke_width(2),
        ))?
        .label("(uparc, downarc)")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                RGBColor(192, 0, 0).stroke_width(2),
            )
        });

    chart
        .draw_series(LineSeries::new(
            (-200..=200)
                .map(|x| x as f32 / 100.0)
                .map(|x| (x, fractal_ease_noise(smooth3, 2, 4, 0.7, x))),
            RGBColor(0, 192, 0).stroke_width(2),
        ))?
        .label("smooth3")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                RGBColor(0, 192, 0).stroke_width(2),
            )
        });

    chart
        .draw_series(LineSeries::new(
            (-200..=200)
                .map(|x| x as f32 / 100.0)
                .map(|x| (x, fractal_ease_noise(id, 108, 4, 0.7, x))),
            RGBColor(96, 96, 96).stroke_width(2),
        ))?
        .label("id")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                RGBColor(96, 96, 96).stroke_width(2),
            )
        });

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    Ok(())
}
