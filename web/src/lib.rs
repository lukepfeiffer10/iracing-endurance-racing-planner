use std::fmt::{Display, Formatter};
use boolinator::Boolinator;
use chrono::{Duration, NaiveDateTime};
use yew::{Component, ComponentLink, Html, ShouldRender};
use yew::prelude::*;
use yew_router::{Switch};
use yew_router::agent::RouteRequest;
use yew_router::prelude::*;
use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;
use crate::bindings::enable_tab_bar;
use crate::event_bus::EventBus;
use crate::overview::fuel_stint_times::{StintData};
use crate::overview::overall_event_config::{EventConfigData};
use crate::overview::overall_fuel_stint_config::OverallFuelStintConfigData;
use crate::overview::Overview;
use crate::overview::per_driver_lap_factors::DriverLapFactor;
use crate::overview::time_of_day_lap_factors::TimeOfDayLapFactor;
use crate::roster::{Driver, DriverRoster};
use crate::schedule::fuel_stint_schedule::ScheduleDataRow;
use crate::schedule::Schedule;
use crate::landing::Landing;

mod bindings;
mod md_text_field;
mod event_bus;
mod duration_serde;
mod overview;
mod roster;
mod schedule;
mod landing;

#[derive(Switch,Clone,Eq,PartialEq)]
enum AppRoutes {
    #[to = "/schedule"]
    Schedule,
    #[to = "/roster"]
    Roster,
    #[to = "/overview"]
    Overview,
    #[to = "/"]
    Landing,
}

impl AppRoutes {
    fn render_tab(&self, is_active: bool) -> Html {
        let icon = match self {
            AppRoutes::Schedule => "schedule",
            AppRoutes::Roster => "list",
            AppRoutes::Overview => "home",
            _ => "N/A"
        };
        
        html! {
            <RouterButton<AppRoutes> route=self.clone()
                classes={classes!["mdc-tab", is_active.as_some("mdc-tab--active")].to_string()}>
                <span class="mdc-tab__content">
                    <span class="mdc-tab__icon material-icons" aria-hidden="true">{ icon }</span>
                    <span class="mdc-tab__text-label">{ format!("{}", self) }</span>
                </span>
                <span class={classes!["mdc-tab-indicator",is_active.as_some("mdc-tab-indicator--active")]}>
                    <span class="mdc-tab-indicator__content mdc-tab-indicator__content--underline"></span>
                </span>
                <span class="mdc-tab__ripple"></span>
            </RouterButton<AppRoutes>> 
        }
    }
}

impl Display for AppRoutes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppRoutes::Schedule => write!(f, "Schedule"),
            AppRoutes::Roster => write!(f, "Roster"),
            AppRoutes::Overview => write!(f, "Overview"),
            AppRoutes::Landing => write!(f, "Landing"),
        }
    }
}

enum AppMsg {
    ChangeRoute(Route)
}

struct App {
    _route_agent_bridge: RouteAgentBridge,
    current_route: AppRoutes,
    _event_bus_bridge: Box<dyn Bridge<EventBus>>,
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut route_agent_bridge = RouteAgentBridge::new(link.callback(AppMsg::ChangeRoute));
        route_agent_bridge.send(RouteRequest::GetCurrentRoute);
        let event_bus_bridge = EventBus::bridge(link.batch_callback(|message| {
            match message {
                _ => None
            }
        }));
        Self {
            _route_agent_bridge: route_agent_bridge,
            current_route: AppRoutes::Overview,
            _event_bus_bridge: event_bus_bridge
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            AppMsg::ChangeRoute(route) => {
                self.current_route = AppRoutes::switch(route).unwrap_or(AppRoutes::Overview);
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="wrapper flex-container flex-column">
                <div class="content">
                    <Router<AppRoutes> render=Router::render(Self::switch) />
                </div>
                <footer>
                    <div class="mdc-tab-bar" role="tablist">
                        <div class="mdc-tab-scroller">
                            <div class="mdc-tab-scroller__scroll-area">
                              <div class="mdc-tab-scroller__scroll-content">                                
                                { AppRoutes::Overview.render_tab(self.is_active_tab(AppRoutes::Overview)) }
                                { AppRoutes::Schedule.render_tab(self.is_active_tab(AppRoutes::Schedule)) }                                
                                { AppRoutes::Roster.render_tab(self.is_active_tab(AppRoutes::Roster)) }
                              </div>
                            </div>
                        </div>
                    </div>
                </footer>
            </div>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            enable_tab_bar(".mdc-tab-bar");
        }
    }
}

impl App {
    fn switch(switch: AppRoutes) -> Html {
        match switch {
            AppRoutes::Roster => {
                return html! {
                    <div class="mdc-typography flex-container flex-row">
                        <DriverRoster />
                    </div>
                }
            }
            AppRoutes::Overview => {
                return html! { 
                    <Overview />
                }
            }
            AppRoutes::Schedule => {
                return html! {
                    <Schedule />
                }
            }
            AppRoutes::Landing => {
                return html! {
                    <Landing />
                }
            }
        }
    }
    
    fn is_active_tab(&self, tab: AppRoutes) -> bool {
        tab == self.current_route        
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuelStintAverageTimes {
    pub standard_fuel_stint: StintData,
    pub fuel_saving_stint: StintData
}

struct RacePlanner {
    overall_event_config: Option<EventConfigData>,
    overall_fuel_stint_config: OverallFuelStintConfigData,
    fuel_stint_average_times: Option<FuelStintAverageTimes>,
    time_of_day_lap_factors: Vec<TimeOfDayLapFactor>,
    per_driver_lap_factors: Vec<DriverLapFactor>,
    driver_roster: Vec<Driver>,
    schedule_rows: Option<Vec<ScheduleDataRow>>
}

impl RacePlanner {
    fn new() -> Self {
        Self {
            overall_event_config: None,
            overall_fuel_stint_config: OverallFuelStintConfigData::new(),
            fuel_stint_average_times: None,
            time_of_day_lap_factors: vec![],
            per_driver_lap_factors: vec![],
            driver_roster: vec![],
            schedule_rows: None
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
        DurationFormat::HourMinSec => {
            let prefix = if duration.num_milliseconds().is_negative() {
                "-"
            } else {
                ""
            };
            format!("{}{:02}:{:02}:{:02}", prefix, duration.num_hours().abs(), duration.num_minutes().abs() % 60, duration.num_seconds().abs() % 60)
        },
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

#[wasm_bindgen(start)]
pub fn main() {
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