use std::fmt::Display;

use chrono::{DateTime, Utc};
use spis_model::MediaListParams;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActiveFilter {
    favorite: Option<bool>,
    timespan: Option<ActiveFilterTimespan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveFilterTimespan {
    year: u16,
    month: Option<u16>,
}

impl ActiveFilter {
    pub fn nothing_set(&self) -> bool {
        self.eq(&Self::default())
    }

    pub fn year(&self) -> Option<u16> {
        self.timespan.as_ref().map(|t| t.year)
    }

    pub fn add(&self, element: &FilterElement) -> Self {
        let mut res = self.clone();
        match element {
            FilterElement::Favorite => res.favorite = Some(true),
            FilterElement::Year(year) => {
                res.timespan = Some(ActiveFilterTimespan {
                    year: *year,
                    month: None,
                })
            }
            FilterElement::Month(year, month) => {
                res.timespan = Some(ActiveFilterTimespan {
                    year: *year,
                    month: Some(*month),
                })
            }
        }
        res
    }

    pub fn remove(&self, element: &FilterElement) -> Self {
        let mut res = self.clone();
        match element {
            FilterElement::Favorite => res.favorite = None,
            FilterElement::Year(_) => res.timespan = None,
            FilterElement::Month(year, _) => {
                res.timespan = Some(ActiveFilterTimespan {
                    year: *year,
                    month: None,
                })
            }
        }
        res
    }

    pub fn toggle(&self, element: &FilterElement) -> Self {
        let new = self.add(element);
        if self.eq(&new) {
            self.remove(element)
        } else {
            new
        }
    }

    pub fn is_active(&self, element: &FilterElement) -> bool {
        self.eq(&self.add(element))
    }
}

impl ActiveFilterTimespan {
    fn get_start(&self) -> DateTime<Utc> {
        Self::to_timestamp(&format!(
            "{}-{:02}-01T00:00:00-00:00",
            self.year,
            self.month.unwrap_or(1)
        ))
    }
    fn get_end(&self) -> DateTime<Utc> {
        let next_year = format!("{}-01-01T00:00:00-00:00", self.year + 1);
        Self::to_timestamp(&match self.month {
            None => next_year,
            Some(month) => {
                if month == 12 {
                    next_year
                } else {
                    format!("{}-{:02}-01T00:00:00-00:00", self.year, month + 1,)
                }
            }
        })
    }
    fn to_timestamp(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s)
            .expect("malformed timestamp")
            .with_timezone(&Utc)
    }
}

impl From<&ActiveFilter> for MediaListParams {
    fn from(f: &ActiveFilter) -> Self {
        MediaListParams {
            favorite: f.favorite,
            taken_after: f.timespan.as_ref().map(|t| t.get_start()),
            taken_before: f.timespan.as_ref().map(|t| t.get_end()),
            ..Default::default()
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum FilterElement {
    Favorite,
    Year(u16),
    Month(u16, u16),
}

impl Display for FilterElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterElement::Favorite => f.write_str("fav"),
            FilterElement::Year(year) => f.write_fmt(format_args!("{}", year)),
            FilterElement::Month(_, month) => match month {
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
        }
    }
}
