use std::fmt::Display;

use chrono::{DateTime, Utc};
use log::info;
use spis_model::MediaListParams;

#[derive(Clone, PartialEq)]
pub enum GuiFilter {
    Favorite,
    Time(GuiFilterTime),
}

#[derive(Clone, PartialEq)]
pub struct GuiFilterTime {
    pub year: u16,
    pub month: Option<u16>,
}

impl GuiFilterTime {
    fn get_datetime(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        let (before, after) = match self.month {
            Some(month) => (
                if month == 12 {
                    format!("{}-01-01T00:00:00-00:00", self.year + 1)
                } else {
                    format!("{}-{:02}-01T00:00:00-00:00", self.year, month + 1)
                },
                format!("{}-{:02}-01T00:00:00-00:00", self.year, month),
            ),
            None => (
                format!("{}-01-01T00:00:00-00:00", self.year + 1),
                format!("{}-01-01T00:00:00-00:00", self.year),
            ),
        };

        let before = DateTime::parse_from_rfc3339(&before)
            .expect("malformed timestamp")
            .with_timezone(&Utc);

        let after = DateTime::parse_from_rfc3339(&after)
            .expect("malformed timestamp")
            .with_timezone(&Utc);

        info!("before: {}", before);
        info!("after: {}", after);

        (before, after)
    }

    pub fn get_subfilters(&self) -> Vec<GuiFilter> {
        let mut res = vec![];
        if self.month.is_some() {
            panic!("this filter has no subfilters");
        }
        for m in 1..=12 {
            res.push(GuiFilter::Time(GuiFilterTime {
                year: self.year,
                month: Some(m),
            }))
        }
        res
    }
}

impl Display for GuiFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuiFilter::Favorite => f.write_str("fav"),
            GuiFilter::Time(time) => match time.month {
                None => f.write_fmt(format_args!("{}", time.year)),
                Some(month) => match month {
                    1 => f.write_fmt(format_args!("Jan")),
                    2 => f.write_fmt(format_args!("Feb")),
                    3 => f.write_fmt(format_args!("Mar")),
                    4 => f.write_fmt(format_args!("Apr")),
                    5 => f.write_fmt(format_args!("May")),
                    6 => f.write_fmt(format_args!("Jun")),
                    7 => f.write_fmt(format_args!("Jul")),
                    8 => f.write_fmt(format_args!("Aug")),
                    9 => f.write_fmt(format_args!("Sep")),
                    10 => f.write_fmt(format_args!("Oct")),
                    11 => f.write_fmt(format_args!("Nov")),
                    12 => f.write_fmt(format_args!("Dec")),
                    _ => unreachable!(),
                },
            },
        }
    }
}

impl From<&GuiFilter> for MediaListParams {
    fn from(value: &GuiFilter) -> Self {
        match value {
            GuiFilter::Favorite => Self {
                favorite: Some(true),
                ..Default::default()
            },
            GuiFilter::Time(time) => {
                let (taken_before, taken_after) = time.get_datetime();
                Self {
                    taken_after: Some(taken_after),
                    taken_before: Some(taken_before),
                    ..Default::default()
                }
            }
        }
    }
}
