use std::fmt::{Display, Formatter};
use boolinator::Boolinator;
use chrono::{Duration, NaiveDateTime};
use yew::{Component, Context, Html};
use yew::prelude::*;
use yew_router::{Switch};
use yew_router::prelude::*;
use yew_agent::{Bridge, Bridged};
use serde::{Serialize, Deserialize};
use yew::html::Scope;
use yew_router::scope_ext::HistoryHandle;
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

#[derive(Routable,Clone,Eq,PartialEq,Copy)]
pub enum PlannerRoutes {
    #[at("/planner/schedule")]
    Schedule,
    #[at("/planner/roster")]
    Roster,
    #[at("/planner/overview")]
    Overview
}

impl PlannerRoutes {
    fn render_tab(&self, is_active: bool, link: &Scope<Planner>) -> Html {
        let icon = match self {
            PlannerRoutes::Schedule => "schedule",
            PlannerRoutes::Roster => "list",
            PlannerRoutes::Overview => "home"
        };

        let tab_route = *self;
        let tab_onclick = link.callback(move |_| PlannerMsg::ChangeRoute(tab_route));

        html! {
            <button onclick={tab_onclick}
                class={classes!["mdc-tab", is_active.as_some("mdc-tab--active")].to_string()}
                role="tab">
                <span class="mdc-tab__content">
                    <span class="mdc-tab__icon material-icons" aria-hidden="true">{ icon }</span>
                    <span class="mdc-tab__text-label">{ format!("{}", self) }</span>
                </span>
                <span class={classes!["mdc-tab-indicator",is_active.as_some("mdc-tab-indicator--active")]}>
                    <span class="mdc-tab-indicator__content mdc-tab-indicator__content--underline"></span>
                </span>
                <span class="mdc-tab__ripple"></span>
            </button>
        }
    }
}

impl Display for PlannerRoutes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PlannerRoutes::Schedule => write!(f, "Schedule"),
            PlannerRoutes::Roster => write!(f, "Roster"),
            PlannerRoutes::Overview => write!(f, "Overview")
        }
    }
}

pub enum PlannerMsg {
    ChangeRoute(PlannerRoutes),
    UpdateTab
}

pub struct Planner {
    _event_bus_bridge: Box<dyn Bridge<EventBus>>,
    _route_listener: HistoryHandle
}

impl Component for Planner {
    type Message = PlannerMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let event_bus_bridge = EventBus::bridge(ctx.link().batch_callback(|_| None));
        let route_listener = ctx.link()
            .add_history_listener(ctx.link().callback(|_| PlannerMsg::UpdateTab))
            .unwrap();
        Self {
            _event_bus_bridge: event_bus_bridge,
            _route_listener: route_listener
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PlannerMsg::ChangeRoute(route) => {
                ctx.link().history().unwrap().push(route);
                false
            }
            PlannerMsg::UpdateTab => {
                true
            }
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let current_route = link.route::<PlannerRoutes>().unwrap();
        html! {
            <>
                <div class="content">
                    <Switch<PlannerRoutes> render={Switch::render(Self::switch)} />
                </div>
                <footer>
                    <div class="mdc-tab-bar" role="tablist">
                        <div class="mdc-tab-scroller">
                            <div class="mdc-tab-scroller__scroll-area">
                              <div class="mdc-tab-scroller__scroll-content">                                
                                { PlannerRoutes::Overview.render_tab(is_active_tab(&current_route, PlannerRoutes::Overview), link) }
                                { PlannerRoutes::Schedule.render_tab(is_active_tab(&current_route, PlannerRoutes::Schedule), link) }                                
                                { PlannerRoutes::Roster.render_tab(is_active_tab(&current_route, PlannerRoutes::Roster), link) }
                              </div>
                            </div>
                        </div>
                    </div>
                </footer>
            </>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            enable_tab_bar(".mdc-tab-bar");
        }
    }
}

impl Planner {
    fn switch(switch: &PlannerRoutes) -> Html {
        match switch {
            PlannerRoutes::Roster => {
                return html! {
                    <div class="mdc-typography flex-container flex-row">
                        <DriverRoster />
                    </div>
                }
            }
            PlannerRoutes::Overview => {
                return html! { 
                    <Overview />
                }
            }
            PlannerRoutes::Schedule => {
                return html! {
                    <Schedule />
                }
            }
        }
    }
}

fn is_active_tab(current_route: &PlannerRoutes, tab: PlannerRoutes) -> bool {
    tab == *current_route
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuelStintAverageTimes {
    pub standard_fuel_stint: StintData,
    pub fuel_saving_stint: StintData
}

pub struct RacePlanner {
    pub overall_event_config: Option<EventConfigData>,
    pub overall_fuel_stint_config: OverallFuelStintConfigData,
    pub fuel_stint_average_times: Option<FuelStintAverageTimes>,
    pub time_of_day_lap_factors: Vec<TimeOfDayLapFactor>,
    pub per_driver_lap_factors: Vec<DriverLapFactor>,
    pub driver_roster: Vec<Driver>,
    pub schedule_rows: Option<Vec<ScheduleDataRow>>
}

impl RacePlanner {
    pub fn new() -> Self {
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

pub const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

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

pub fn format_date_time(date_time: NaiveDateTime) -> String {
    date_time.format(DATE_FORMAT).to_string()
}

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

pub fn parse_duration_from_str(str: &str, format: DurationFormat) -> Result<Duration, &str> {
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