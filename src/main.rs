use chrono::{Duration, NaiveDateTime};
use yew::{Component, ComponentLink, Html, ShouldRender};
use yew::prelude::*;
use yew_router::{Switch};
use yew_router::prelude::*;
use crate::overview::Overview;

mod bindings;
mod md_text_field;
mod event_bus;
mod duration_serde;
mod overview;

#[derive(Switch,Clone)]
enum AppRoutes {
    #[to = "/roster"]
    Roster,
    #[to = "/"]
    Overview,
}

struct App;

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <Router<AppRoutes> render=Router::render(Self::switch) />
            </>
        }
    }
}

impl App {
    fn switch(switch: AppRoutes) -> Html {
        match switch {
            AppRoutes::Roster => {
                return html! {
                    <p> { "Work in Progress" }</p>
                }
            }
            AppRoutes::Overview => {
                return html! { 
                    <Overview />
                }
            }
        }
    }
}

const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub enum DurationFormat {
    HourMinSec,
    MinSecMilli
}

pub fn format_duration(duration: Duration, format: DurationFormat) -> String {
    match format {
        DurationFormat::HourMinSec => format!("{:02}:{:02}:{:02}", duration.num_hours(), duration.num_minutes() % 60, duration.num_seconds() % 60),
        DurationFormat::MinSecMilli => {
            let prefix = if duration.num_milliseconds().is_negative() {
                "-"
            } else {
                ""
            };
            format!("{}{:02}:{:02}.{:03}", prefix, duration.num_minutes().abs() % 60, duration.num_seconds().abs() % 60, duration.num_milliseconds().abs() % 1000)
        }
    }
}

fn format_date_time(date_time: NaiveDateTime) -> String {
    date_time.format(DATE_FORMAT).to_string()
}

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

fn parse_duration_from_str(str: &str, format: DurationFormat) -> Result<Duration, &str> {
    let duration = match format {
       DurationFormat::HourMinSec => {
           let duration_split = str
               .split(':')
               .into_iter()
               .map(|part| part.parse::<i64>().unwrap())
               .collect::<Vec<_>>();

           let duration_seconds = match duration_split.len() {
               3 => {
                   Some((duration_split[0] * 60 + duration_split[1]) * 60 + duration_split[2])
               }
               2 => {
                   Some(duration_split[0] * 60 + duration_split[1])
               }
               1 => {
                   Some(duration_split[0])
               }
               _ => None
           };
           
           duration_seconds
               .map(|value| Duration::seconds(value))
       }
       DurationFormat::MinSecMilli => {
           let pattern = regex!(r"(?P<M1>\d{1,2}):(?P<S1>\d{1,2}).(?P<mil1>\d+)|(?P<M2>\d{1,2}):(?P<S2>\d{1,2})|(?P<S3>\d{1,2}).(?P<mil2>\d+)|(?P<S4>\d+)");
           let captures = pattern.captures(str);
           captures.map(|captures| {               
               let minutes = captures.name("M1")
                   .or(captures.name("M2"))
                   .and_then(|m| m.as_str().parse::<i64>().ok())
                   .unwrap_or(0);
               let seconds = captures.name("S1")
                   .or(captures.name("S2"))
                   .or(captures.name("S3"))
                   .or(captures.name("S4"))
                   .and_then(|m| m.as_str().parse::<i64>().ok())
                   .unwrap();
               let milliseconds = captures.name("mil1")
                   .or(captures.name("mil2"))
                   .map(|m| {
                       let mut milliseconds = m.as_str().parse::<i64>().unwrap();
                       if milliseconds < 10 {
                           milliseconds *= 10;
                       }
                       
                       if milliseconds < 100 {
                           milliseconds *= 10;
                       }
                       
                       milliseconds
                   })
                   .unwrap_or(0);
               
               Duration::milliseconds((minutes * 60 + seconds) * 1000 + milliseconds)
           })
       }
    };
    
    duration.ok_or("the duration could not be parsed")
}

fn main() {
    yew::start_app::<App>();
}

#[cfg(test)]
mod parse_hour_min_sec_duration {
    use super::{parse_duration_from_str, DurationFormat};
    use chrono::Duration;

    #[test]
    fn should_parse_as_hours_minutes_seconds_with_three_parts() {
        let ten_hours_str = "10:00:00";
        let parsed = parse_duration_from_str(ten_hours_str, DurationFormat::HourMinSec);
        
        assert!(parsed.is_ok());
        assert_eq!(Duration::hours(10), parsed.unwrap());
    }

    #[test]
    fn should_parse_as_minutes_seconds_with_only_two_parts() {
        let ten_minutes_str = "10:00";
        let parsed = parse_duration_from_str(ten_minutes_str, DurationFormat::HourMinSec);

        assert!(parsed.is_ok());
        assert_eq!(Duration::minutes(10), parsed.unwrap());
    }

    #[test]
    fn should_parse_as_seconds_with_only_one_part() {
        let ten_seconds_str = "10";
        let parsed = parse_duration_from_str(ten_seconds_str, DurationFormat::HourMinSec);

        assert!(parsed.is_ok());
        assert_eq!(Duration::seconds(10), parsed.unwrap());
    }
}

#[cfg(test)]
mod parse_min_sec_milli_duration {
    use super::{parse_duration_from_str, DurationFormat};

    #[test]
    fn should_parse_as_min_sec_milli_with_three_parts() {
        let value = "1:16.2";
        let parsed = parse_duration_from_str(value, DurationFormat::MinSecMilli);

        assert!(parsed.is_ok());
        assert_eq!(1, parsed.unwrap().num_minutes());
        assert_eq!(16, parsed.unwrap().num_seconds() % 60);
        assert_eq!(200, parsed.unwrap().num_milliseconds() % 1000);
    }

    #[test]
    fn should_parse_as_min_sec_with_no_milliseconds_present() {
        let value = "1:16";
        let parsed = parse_duration_from_str(value, DurationFormat::MinSecMilli);

        assert!(parsed.is_ok());
        assert_eq!(1, parsed.unwrap().num_minutes());
        assert_eq!(16, parsed.unwrap().num_seconds() % 60);
        assert_eq!(0, parsed.unwrap().num_milliseconds() % 1000);
    }

    #[test]
    fn should_parse_as_sec_milli_with_milliseconds_present() {
        let value = "16.200";
        let parsed = parse_duration_from_str(value, DurationFormat::MinSecMilli);

        assert!(parsed.is_ok());
        assert_eq!(0, parsed.unwrap().num_minutes());
        assert_eq!(16, parsed.unwrap().num_seconds() % 60);
        assert_eq!(200, parsed.unwrap().num_milliseconds() % 1000);
    }

    #[test]
    fn should_parse_as_seconds_with_only_one_part() {
        let value = "16";
        let parsed = parse_duration_from_str(value, DurationFormat::MinSecMilli);

        assert!(parsed.is_ok());
        assert_eq!(0, parsed.unwrap().num_minutes());
        assert_eq!(16, parsed.unwrap().num_seconds() % 60);
        assert_eq!(0, parsed.unwrap().num_milliseconds() % 1000);
    }
}