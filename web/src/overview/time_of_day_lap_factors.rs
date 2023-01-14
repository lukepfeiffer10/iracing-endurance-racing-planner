use crate::md_text_field::{
    MaterialTextField, MaterialTextFieldIcon, MaterialTextFieldIconStyle, MaterialTextFieldProps,
};
use crate::planner::{format_duration, parse_duration_from_str, DurationFormat};
use boolinator::Boolinator;
use chrono::{Duration, NaiveTime};
use gloo_console::error;
use yew::html::Scope;
use yew::{classes, html, props, Component, Context, Html, Properties};

const TIME_OF_DAY_TIME_FORMAT: &str = "%I:%M %p";

#[derive(Debug, PartialEq, Clone)]
pub struct TimeOfDayLapFactor {
    time_of_day: String,
    lap_time: Duration,
    tod_start: NaiveTime,
    delta: Duration,
    factor: f64,
    has_edited_lap_time: bool,
}

impl Eq for TimeOfDayLapFactor {}

impl TimeOfDayLapFactor {
    fn set_lap_time_if_unset(&mut self, lap_time: Duration) {
        if !self.has_edited_lap_time {
            self.lap_time = lap_time;
        }
    }

    fn update_lap_time(&mut self, lap_time: Duration, reference_lap_time: Duration) {
        self.lap_time = lap_time;
        self.has_edited_lap_time = true;
        self.compute_factor_and_delta_from_reference(reference_lap_time);
    }

    fn reset_lap_time(&mut self, reference_lap_time: Duration) {
        self.lap_time = reference_lap_time;
        self.has_edited_lap_time = false;
        self.factor = 1.0;
        self.delta = Duration::zero();
    }

    fn compute_factor_and_delta_from_reference(&mut self, reference: Duration) {
        self.factor =
            (self.lap_time.num_milliseconds() as f64) / (reference.num_milliseconds() as f64);
        self.delta = self.lap_time - reference;
    }
}

pub enum TimeOfDayLapFactorsMsg {
    UpdateLapTime(String, usize),
    UpdateTimeOfDay(String, usize),
    UpdateTimeOfDayStart(String, usize),
    ResetLapTimeToReference(usize),
}

#[derive(Properties, PartialEq)]
pub struct TimeOfDayLapFactorsProps {
    #[prop_or(Duration::zero())]
    pub lap_time: Duration,
}

pub struct TimeOfDayLapFactors {
    factors: Vec<TimeOfDayLapFactor>,
    reference_lap_time: Duration,
}

impl Component for TimeOfDayLapFactors {
    type Message = TimeOfDayLapFactorsMsg;
    type Properties = TimeOfDayLapFactorsProps;

    fn create(ctx: &Context<Self>) -> Self {
        let Self::Properties { lap_time } = ctx.props();

        let factors = vec![
            TimeOfDayLapFactor {
                time_of_day: "Night1".to_string(),
                lap_time: lap_time.clone(),
                tod_start: NaiveTime::from_hms(0, 0, 0),
                delta: Duration::zero(),
                factor: 1.0,
                has_edited_lap_time: false,
            },
            TimeOfDayLapFactor {
                time_of_day: "Morning".to_string(),
                lap_time: lap_time.clone(),
                tod_start: NaiveTime::from_hms(4, 0, 0),
                delta: Duration::zero(),
                factor: 1.0,
                has_edited_lap_time: false,
            },
            TimeOfDayLapFactor {
                time_of_day: "Afternoon".to_string(),
                lap_time: lap_time.clone(),
                tod_start: NaiveTime::from_hms(11, 0, 0),
                delta: Duration::zero(),
                factor: 1.0,
                has_edited_lap_time: false,
            },
            TimeOfDayLapFactor {
                time_of_day: "Evening".to_string(),
                lap_time: lap_time.clone(),
                tod_start: NaiveTime::from_hms(18, 0, 0),
                delta: Duration::zero(),
                factor: 1.0,
                has_edited_lap_time: false,
            },
            TimeOfDayLapFactor {
                time_of_day: "Night2".to_string(),
                lap_time: lap_time.clone(),
                tod_start: NaiveTime::from_hms(21, 0, 0),
                delta: Duration::zero(),
                factor: 1.0,
                has_edited_lap_time: false,
            },
        ];
        Self {
            factors,
            reference_lap_time: lap_time.clone(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TimeOfDayLapFactorsMsg::UpdateLapTime(lap_time, index) => {
                let factor = &mut self.factors[index];
                let parsed_lap_time =
                    parse_duration_from_str(lap_time.as_str(), DurationFormat::MinSecMilli);

                match parsed_lap_time {
                    Ok(lap_time) => {
                        factor.update_lap_time(lap_time, self.reference_lap_time);
                        true
                    }
                    Err(e) => {
                        error!(format!("the factor lap time parse failed: {}", e).as_str());
                        false
                    }
                }
            }
            TimeOfDayLapFactorsMsg::UpdateTimeOfDay(value, index) => {
                let factor = &mut self.factors[index];
                factor.time_of_day = value;
                false
            }
            TimeOfDayLapFactorsMsg::UpdateTimeOfDayStart(value, index) => {
                let factor = &mut self.factors[index];
                let parsed_time =
                    NaiveTime::parse_from_str(value.as_str(), TIME_OF_DAY_TIME_FORMAT);
                match parsed_time {
                    Ok(time) => {
                        factor.tod_start = time;
                        true
                    }
                    Err(e) => {
                        error!(format!("the time of day start time parse failed: {:?}", e).as_str());
                        false
                    }
                }
            }
            TimeOfDayLapFactorsMsg::ResetLapTimeToReference(index) => {
                let factor = &mut self.factors[index];
                factor.reset_lap_time(self.reference_lap_time);
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let Self::Properties { lap_time } = ctx.props();
        self.reference_lap_time = *lap_time;
        for f in &mut self.factors {
            f.set_lap_time_if_unset(*lap_time);
            f.compute_factor_and_delta_from_reference(*lap_time);
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div id="time-of-day-lap-factors" class="mdc-card">
                <div class="mdc-card-wrapper__text-section">
                    <div class="card-title">{ "Time of Day Lap Factors" }</div>
                </div>
                <div class="mdc-data-table">
                  <div class="mdc-data-table__table-container">
                    <table class="mdc-data-table__table">
                      <thead>
                        <tr class="mdc-data-table__header-row">
                          <th class="mdc-data-table__header-cell" role="columnheader" scope="col">{ "Time of Day" }</th>
                          <th class="mdc-data-table__header-cell" role="columnheader" scope="col">{ "Laptime  (MM:SS.mmm)" }</th>
                          <th class="mdc-data-table__header-cell" role="columnheader" scope="col">{ "ToD Start" }</th>
                          <th class="mdc-data-table__header-cell" role="columnheader" scope="col">{ "Delta" }</th>
                          <th class="mdc-data-table__header-cell mdc-data-table__header-cell--numeric" role="columnheader" scope="col">{ "Factor" }</th>
                        </tr>
                      </thead>
                      <tbody class="mdc-data-table__content">
                        {
                            self.factors
                                .iter()
                                .enumerate()
                                .map(|(index,f)| render_time_of_day_lap_factor(f, ctx.link(), index))
                                .collect::<Vec<_>>()
                        }
                      </tbody>
                    </table>
                  </div>
                </div>
            </div>
        }
    }
}

fn render_time_of_day_lap_factor(
    factor: &TimeOfDayLapFactor,
    link: &Scope<TimeOfDayLapFactors>,
    index: usize,
) -> Html {
    let time_of_day_props = props! {MaterialTextFieldProps {
        value: factor.time_of_day.clone(),
        on_change: link.callback(move |value| TimeOfDayLapFactorsMsg::UpdateTimeOfDay(value, index)),
    }};
    let lap_time_props = props! {MaterialTextFieldProps {
        value: format_duration(factor.lap_time, DurationFormat::MinSecMilli),
        on_change: link.callback(move |value| TimeOfDayLapFactorsMsg::UpdateLapTime(value, index)),
        end_aligned: true,
        icon: Some(MaterialTextFieldIcon {
            style: MaterialTextFieldIconStyle::Leading,
            icon: "restart_alt".to_string(),
            on_click: Some(link.callback(move |_| TimeOfDayLapFactorsMsg::ResetLapTimeToReference(index))),
            background_color: None
        })
    }};
    let tod_start_props = props! {MaterialTextFieldProps {
        value: factor.tod_start.format(TIME_OF_DAY_TIME_FORMAT).to_string(),
        on_change: link.callback(move |value| TimeOfDayLapFactorsMsg::UpdateTimeOfDayStart(value, index)),
        end_aligned: true
    }};
    html! {
        <tr class="mdc-data-table__row">
          <td class="mdc-data-table__cell">
            <MaterialTextField ..time_of_day_props />
          </td>
          <td class={classes!("mdc-data-table__cell", factor.has_edited_lap_time.as_some("show-reset"))}>
            <MaterialTextField ..lap_time_props />
          </td>
          <td class="mdc-data-table__cell">
            <MaterialTextField ..tod_start_props />
          </td>
          <td class="mdc-data-table__cell">{ format_duration(factor.delta, DurationFormat::MinSecMilli) }</td>
          <td class="mdc-data-table__cell mdc-data-table__cell--numeric">{ format!("{:.2}", factor.factor) }</td>
        </tr>
    }
}
