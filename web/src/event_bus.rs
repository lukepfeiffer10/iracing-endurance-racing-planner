use crate::planner::RacePlanner;
use endurance_racing_planner_common::Driver;
use gloo_console::log;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use yew_agent::{Agent, AgentLink, Context, HandlerId};

#[derive(Serialize, Deserialize, Debug)]
pub enum EventBusInput {
    GetDriverRoster,
    PutDriverRoster(Vec<Driver>),
    PutPlannerTitle(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EventBusOutput {
    SendDriverRoster(Vec<Driver>),
    SendPlannerTitle(String),
}

pub struct EventBus {
    link: AgentLink<EventBus>,
    subscribers: HashSet<HandlerId>,
    race_planner_data: RacePlanner,
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
            race_planner_data: RacePlanner::new(),
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            EventBusInput::GetDriverRoster => {
                for sub in self.subscribers.iter() {
                    self.link.respond(
                        *sub,
                        EventBusOutput::SendDriverRoster(
                            self.race_planner_data.data.driver_roster.clone(),
                        ),
                    )
                }
            }
            EventBusInput::PutDriverRoster(drivers) => {
                self.race_planner_data.data.driver_roster = drivers;
            }
            EventBusInput::PutPlannerTitle(title) => {
                for sub in self.subscribers.iter() {
                    self.link
                        .respond(*sub, EventBusOutput::SendPlannerTitle(title.clone()))
                }
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
