use super::utils::ngram_time_to_date;
use crate::processor::ngram::NgramsForByContentCommand;
use image::{ImageError, ImageFormat, RgbImage};
use plotters::{
    backend::BitMapBackend,
    chart::ChartBuilder,
    drawing::{DrawingAreaErrorKind, IntoDrawingArea},
    series::LineSeries,
    style::full_palette::{RED, WHITE},
};
use std::io::Cursor;

const CHART_WIDTH: u32 = 1920;
const CHART_HEIGHT: u32 = 1080;

#[derive(Debug)]
pub enum Error {
    DrawingError(
        DrawingAreaErrorKind<
            <plotters::prelude::BitMapBackend<'static> as plotters::prelude::DrawingBackend>::ErrorType,
        >,
    ),
    ImageError(ImageError),
    InvalidParameter(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DrawingError(err) => write!(f, "Drawing error: {}", err),
            Self::ImageError(err) => write!(f, "Image error: {}", err),
            Self::InvalidParameter(err) => write!(f, "Invalid parameter: {}", err),
        }
    }
}

impl
    From<
        DrawingAreaErrorKind<
            <plotters::prelude::BitMapBackend<'_> as plotters::prelude::DrawingBackend>::ErrorType,
        >,
    > for Error
{
    fn from(
        err: DrawingAreaErrorKind<
            <plotters::prelude::BitMapBackend<'_> as plotters::prelude::DrawingBackend>::ErrorType,
        >,
    ) -> Self {
        Self::DrawingError(err)
    }
}

impl From<ImageError> for Error {
    fn from(err: ImageError) -> Self {
        Self::ImageError(err)
    }
}

#[allow(clippy::missing_panics_doc)]
pub fn display_ngram_count_over_time(
    ngrams: &[NgramsForByContentCommand],
) -> Result<Vec<u8>, Error> {
    let (first_ngram, last_ngram) = match (ngrams.first(), ngrams.last()) {
        (Some(f), Some(l)) => (f, l),
        _ => {
            return Err(Error::InvalidParameter(String::from(
                "Received an empty ngrams array!",
            )))
        }
    };

    let mut image_buffer = vec![0; CHART_WIDTH as usize * CHART_HEIGHT as usize * 3];
    {
        let drawing_area =
            BitMapBackend::with_buffer(&mut image_buffer, (CHART_WIDTH, CHART_HEIGHT))
                .into_drawing_area();

        drawing_area.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&drawing_area)
            .margin(50)
            .x_label_area_size(100)
            .y_label_area_size(100)
            .build_cartesian_2d(
                ngram_time_to_date(first_ngram.time)..ngram_time_to_date(last_ngram.time),
                0..ngrams.iter().map(|ngram| ngram.count).max().unwrap_or(0),
            )?;
        chart
            .configure_mesh()
            .axis_desc_style(("Ubuntu Medium", 50))
            .x_desc("Time")
            .x_label_style(("Ubuntu Medium", 20))
            .x_labels(15)
            .y_desc("Number of occurrences")
            .y_label_style(("Ubuntu Medium", 20))
            .y_labels(20)
            .draw()?;
        chart.draw_series(LineSeries::new(
            ngrams
                .iter()
                .map(|ngram| (ngram_time_to_date(ngram.time), ngram.count)),
            &RED,
        ))?;

        drawing_area.present()?;
    }

    let mut cursor = Cursor::new(Vec::new());
    #[allow(clippy::unwrap_used)]
    // This cannot return None, if we are using the same correct constants everywhere
    let image = RgbImage::from_raw(CHART_WIDTH, CHART_HEIGHT, image_buffer).unwrap();
    image.write_to(&mut cursor, ImageFormat::Png)?;

    Ok(cursor.into_inner())
}
