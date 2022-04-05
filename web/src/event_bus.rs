use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use gloo_console::log;
use yew_agent::{Agent, AgentLink, Context, HandlerId};
use crate::overview::{overall_event_config::EventConfigData, overall_fuel_stint_config::OverallFuelStintConfigData, fuel_stint_times::StandardLapTime};
use crate::schedule::fuel_stint_schedule::{ScheduleDataRow, ScheduleRelatedData};
use crate::{roster::{Driver}, planner::{FuelStintAverageTimes, RacePlanner}};

#[derive(Serialize, Deserialize, Debug)]
pub enum EventBusInput {
    GetOverallFuelStintConfig,
    OverallFuelStintConfig(OverallFuelStintConfigData),
    StandardLapTime(StandardLapTime),
    GetDriverRoster,
    PutDriverRoster(Vec<Driver>),
    GetOverallEventConfig,
    PutOverallEventConfig(EventConfigData),
    GetFuelStintAverageTimes,
    PutFuelStintAverageTimes(FuelStintAverageTimes),
    GetScheduleAndRelatedData,
    PutSchedule(Vec<ScheduleDataRow>),
    GetPlannerTitle,
    PutPlannerTitle(String)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EventBusOutput {
    OverallFuelStintConfig(OverallFuelStintConfigData),
    SendOverallFuelStintConfig(OverallFuelStintConfigData),
    StandardLapTime(StandardLapTime),
    SendDriverRoster(Vec<Driver>),
    SendOverallEventConfig(Option<EventConfigData>),
    SendFuelStintAverageTimes(Option<FuelStintAverageTimes>),
    SendScheduleAndRelatedData(Option<Vec<ScheduleDataRow>>, ScheduleRelatedData),
    SendPlannerTitle(String)
}

pub struct EventBus {
    link: AgentLink<EventBus>,
    subscribers: HashSet<HandlerId>,
    race_planner_data: RacePlanner
}

impl Agent for EventBus {
    type Reach = Context<Self>;
    type Message = ();
    type Input = EventBusInput;
    type Output = EventBusOutput;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::new(),
            race_planner_data: RacePlanner::new()
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            EventBusInput::OverallFuelStintConfig(data) => {
                self.race_planner_data.overall_fuel_stint_config = data.clone();
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, EventBusOutput::OverallFuelStintConfig(data.clone()));
                }
            }
            EventBusInput::GetOverallFuelStintConfig => {
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, EventBusOutput::SendOverallFuelStintConfig(self.race_planner_data.overall_fuel_stint_config.clone()));
                }
            }
            EventBusInput::StandardLapTime(duration) => {
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, EventBusOutput::StandardLapTime(duration.clone()));
                }
            }
            EventBusInput::GetDriverRoster => {
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, EventBusOutput::SendDriverRoster(self.race_planner_data.driver_roster.clone()))
                }
            }
            EventBusInput::PutDriverRoster(drivers) => {                
                self.race_planner_data.driver_roster = drivers;
            }
            EventBusInput::GetOverallEventConfig => {
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, EventBusOutput::SendOverallEventConfig(self.race_planner_data.overall_event_config.clone()))
                }
            }
            EventBusInput::PutOverallEventConfig(config) => {
                self.race_planner_data.overall_event_config = Some(config);
            }
            EventBusInput::GetFuelStintAverageTimes => {
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, EventBusOutput::SendFuelStintAverageTimes(self.race_planner_data.fuel_stint_average_times.clone()))
                }
            }
            EventBusInput::PutFuelStintAverageTimes(data) => {
                self.race_planner_data.fuel_stint_average_times = Some(data);
            }
            EventBusInput::GetScheduleAndRelatedData => {
                for sub in self.subscribers.iter() {
                    let data = ScheduleRelatedData {
                        overall_event_config: self.race_planner_data.overall_event_config.clone(),
                        fuel_stint_times: self.race_planner_data.fuel_stint_average_times.clone(),
                        overall_fuel_stint_config: self.race_planner_data.overall_fuel_stint_config.clone(),
                        drivers: self.race_planner_data.driver_roster.clone()
                    };
                    self.link.respond(*sub, EventBusOutput::SendScheduleAndRelatedData(self.race_planner_data.schedule_rows.clone(), data));
                }
            }
            EventBusInput::PutSchedule(data) => {
                self.race_planner_data.schedule_rows = Some(data);
            }
            EventBusInput::GetPlannerTitle => {
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, EventBusOutput::SendPlannerTitle(self.race_planner_data.title.clone()))
                }
            }
            EventBusInput::PutPlannerTitle(title) => {
                self.race_planner_data.title = title;
            }
        }
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }

    fn destroy(&mut self) {
        log!("event bus destroyed")
    }
}