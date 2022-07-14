use crate::bindings::enable_tab_bar;
use crate::event_bus::{EventBus, EventBusOutput};
use crate::http::plans::{create_plan, get_plan, patch_plan};
use crate::overview::fuel_stint_times::StintData;
use crate::overview::overall_event_config::EventConfigData;
use crate::overview::overall_fuel_stint_config::OverallFuelStintConfigData;
use crate::overview::per_driver_lap_factors::DriverLapFactor;
use crate::overview::time_of_day_lap_factors::TimeOfDayLapFactor;
use crate::overview::Overview;
use crate::roster::{Driver, DriverRoster};
use crate::schedule::fuel_stint_schedule::ScheduleDataRow;
use crate::schedule::Schedule;
use crate::{AppStateAction, AppStateContext};
use boolinator::Boolinator;
use chrono::{Duration, NaiveDateTime};
use endurance_racing_planner_common::{PatchRacePlannerDto, RacePlannerDto};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use uuid::Uuid;
use yew::html::Scope;
use yew::prelude::*;
use yew::{Component, Context, Html};
use yew_agent::{Bridge, Bridged};
use yew_router::prelude::*;
use yew_router::scope_ext::HistoryHandle;
use yew_router::Switch;

#[derive(Routable, Clone, Eq, PartialEq, Copy)]
pub enum PlannerRoutes {
    #[at("/planner/:id/schedule")]
    Schedule { id: Uuid },
    #[at("/planner/:id/roster")]
    Roster { id: Uuid },
    #[at("/planner/:id/overview")]
    Overview { id: Uuid },
}

fn render_tab(
    tab_route: PlannerRoutes,
    current_route: &PlannerRoutes,
    link: &Scope<Planner>,
) -> Html {
    let icon = match tab_route {
        PlannerRoutes::Schedule { id: _ } => "schedule",
        PlannerRoutes::Roster { id: _ } => "list",
        PlannerRoutes::Overview { id: _ } => "home",
    };

    let is_active = *current_route == tab_route;
    let tab_onclick = link.callback(move |_| PlannerMsg::ChangeRoute(tab_route));

    html! {
        <button onclick={tab_onclick}
            class={classes!["mdc-tab", is_active.as_some("mdc-tab--active")].to_string()}
            role="tab">
            <span class="mdc-tab__content">
                <span class="mdc-tab__icon material-icons" aria-hidden="true">{ icon }</span>
                <span class="mdc-tab__text-label">{ format!("{}", &tab_route) }</span>
            </span>
            <span class={classes!["mdc-tab-indicator",is_active.as_some("mdc-tab-indicator--active")]}>
                <span class="mdc-tab-indicator__content mdc-tab-indicator__content--underline"></span>
            </span>
            <span class="mdc-tab__ripple"></span>
        </button>
    }
}

impl Display for PlannerRoutes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PlannerRoutes::Schedule { id: _ } => write!(f, "Schedule"),
            PlannerRoutes::Roster { id: _ } => write!(f, "Roster"),
            PlannerRoutes::Overview { id: _ } => write!(f, "Overview"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct PlannerContext {
    pub data: Rc<RacePlannerDto>,
}

pub enum PlannerMsg {
    ChangeRoute(PlannerRoutes),
    UpdateTab,
    UpdatePlan(RacePlannerDto),
    UpdatePlanTitle(String),
}

pub struct Planner {
    _event_bus_bridge: Box<dyn Bridge<EventBus>>,
    _route_listener: HistoryHandle,
    context: PlannerContext,
}

impl Component for Planner {
    type Message = PlannerMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();
        let event_bus_bridge = EventBus::bridge(link.batch_callback(move |event| {
            if let EventBusOutput::SendPlannerTitle(title) = event {
                Some(PlannerMsg::UpdatePlanTitle(title))
            } else {
                None
            }
        }));
        let route_listener = link
            .add_history_listener(link.callback(|_| PlannerMsg::UpdateTab))
            .unwrap();

        let current_route = link.route::<PlannerRoutes>().unwrap();
        let id = match current_route {
            PlannerRoutes::Schedule { id }
            | PlannerRoutes::Roster { id }
            | PlannerRoutes::Overview { id } => id,
        };

        let mut default_plan = RacePlannerDto::new();
        if Uuid::is_nil(&id) {
            link.history().unwrap().replace(PlannerRoutes::Overview {
                id: default_plan.id,
            });
            create_plan(
                default_plan.clone(),
                link.callback(|plan| PlannerMsg::UpdatePlan(plan)),
            );
        } else {
            default_plan = RacePlannerDto { id, ..default_plan };
            get_plan(id, link.callback(|plan| PlannerMsg::UpdatePlan(plan)));
        }

        Self {
            _event_bus_bridge: event_bus_bridge,
            _route_listener: route_listener,
            context: PlannerContext {
                data: Rc::new(default_plan),
            },
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PlannerMsg::ChangeRoute(route) => {
                ctx.link().history().unwrap().push(route);
                false
            }
            PlannerMsg::UpdateTab => true,
            PlannerMsg::UpdatePlan(plan) => {
                let (app_state_context, _) = ctx
                    .link()
                    .context::<AppStateContext>(Callback::noop())
                    .unwrap();
                app_state_context.dispatch(AppStateAction::SetPageTitle(plan.title.clone()));

                self.context.data = Rc::new(plan);

                true
            }
            PlannerMsg::UpdatePlanTitle(title) => {
                let mut plan_data = Rc::make_mut(&mut self.context.data);
                plan_data.title = title.clone();
                patch_plan(
                    plan_data.id,
                    PatchRacePlannerDto {
                        id: plan_data.id,
                        title: Some(title),
                        overall_event_config: None,
                        overall_fuel_stint_config: None,
                        fuel_stint_average_times: None,
                        time_of_day_lap_factors: None,
                        per_driver_lap_factors: None,
                        driver_roster: None,
                        schedule_rows: None,
                    },
                );
                false
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
                    <ContextProvider<PlannerContext> context={self.context.clone()}>
                        <Switch<PlannerRoutes> render={Switch::render(Self::switch)} />
                    </ContextProvider<PlannerContext>>
                </div>
                <footer>
                    <div class="mdc-tab-bar" role="tablist">
                        <div class="mdc-tab-scroller">
                            <div class="mdc-tab-scroller__scroll-area">
                              <div class="mdc-tab-scroller__scroll-content">
                                { render_tab(PlannerRoutes::Overview { id: self.context.data.id }, &current_route, link) }
                                { render_tab(PlannerRoutes::Schedule { id: self.context.data.id }, &current_route, link) }
                                { render_tab(PlannerRoutes::Roster { id: self.context.data.id }, &current_route, link) }
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
            PlannerRoutes::Roster { id: _ } => {
                return html! {
                    <div class="mdc-typography flex-container flex-row">
                        <DriverRoster />
                    </div>
                }
            }
            PlannerRoutes::Overview { id: _ } => {
                return html! {
                    <Overview />
                }
            }
            PlannerRoutes::Schedule { id: _ } => {
                return html! {
                    <Schedule />
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuelStintAverageTimes {
    pub standard_fuel_stint: StintData,
    pub fuel_saving_stint: StintData,
}

pub struct RacePlanner {
    pub overall_event_config: Option<EventConfigData>,
    pub overall_fuel_stint_config: OverallFuelStintConfigData,
    pub fuel_stint_average_times: Option<FuelStintAverageTimes>,
    pub time_of_day_lap_factors: Vec<TimeOfDayLapFactor>,
    pub per_driver_lap_factors: Vec<DriverLapFactor>,
    pub driver_roster: Vec<Driver>,
    pub schedule_rows: Option<Vec<ScheduleDataRow>>,
    pub title: String,
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
            schedule_rows: None,
            title: "New Plan".to_string(),
        }
    }
}

pub const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub enum DurationFormat {
    HourMinSec,
    MinSecMilli,
}

pub fn format_duration(duration: Duration, format: DurationFormat) -> String {
    match format {
        DurationFormat::HourMinSec => {
            let prefix = if duration.num_milliseconds().is_negative() {
                "-"
            } else {
                ""
            };
            format!(
                "{}{:02}:{:02}:{:02}",
                prefix,
                duration.num_hours().abs(),
                duration.num_minutes().abs() % 60,
                duration.num_seconds().abs() % 60
            )
        }
        DurationFormat::MinSecMilli => {
            let prefix = if duration.num_milliseconds().is_negative() {
                "-"
            } else {
                ""
            };
            format!(
                "{}{:02}:{:02}.{:03}",
                prefix,
                duration.num_minutes().abs() % 60,
                duration.num_seconds().abs() % 60,
                duration.num_milliseconds().abs() % 1000
            )
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
                3 => Some((duration_split[0] * 60 + duration_split[1]) * 60 + duration_split[2]),
                2 => Some(duration_split[0] * 60 + duration_split[1]),
                1 => Some(duration_split[0]),
                _ => None,
            };

            duration_seconds.map(|value| Duration::seconds(value))
        }
        DurationFormat::MinSecMilli => {
            let pattern = regex!(
                r"(?P<M1>\d{1,2}):(?P<S1>\d{1,2}).(?P<mil1>\d+)|(?P<M2>\d{1,2}):(?P<S2>\d{1,2})|(?P<S3>\d{1,2}).(?P<mil2>\d+)|(?P<S4>\d+)"
            );
            let captures = pattern.captures(str);
            captures.map(|captures| {
                let minutes = captures
                    .name("M1")
                    .or(captures.name("M2"))
                    .and_then(|m| m.as_str().parse::<i64>().ok())
                    .unwrap_or(0);
                let seconds = captures
                    .name("S1")
                    .or(captures.name("S2"))
                    .or(captures.name("S3"))
                    .or(captures.name("S4"))
                    .and_then(|m| m.as_str().parse::<i64>().ok())
                    .unwrap();
                let milliseconds = captures
                    .name("mil1")
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
