use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use yew::services::ConsoleService;
use yew::worker::*;
use crate::overview::{overall_fuel_stint_config::OverallFuelStintConfigData, fuel_stint_times::StandardLapTime};
use crate::{Driver, RacePlanner};

#[derive(Serialize, Deserialize, Debug)]
pub enum EventBusInput {
    OverallFuelStintConfig(OverallFuelStintConfigData),
    StandardLapTime(StandardLapTime),
    GetDriverRoster,
    PutDriverRoster(Vec<Driver>)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EventBusOutput {
    OverallFuelStintConfig(OverallFuelStintConfigData),
    StandardLapTime(StandardLapTime),
    SendDriverRoster(Vec<Driver>)
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
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, EventBusOutput::OverallFuelStintConfig(data.clone()));
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
        }
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }

    fn destroy(&mut self) {
        ConsoleService::log("event bus destroyed")
    }
}