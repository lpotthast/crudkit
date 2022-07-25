use chrono::{Datelike, NaiveDateTime};
use chrono_utc_date_time::UtcDateTime;
use yew::prelude::*;

use crate::crud_datetime::{Day, GuideMode, Month, Week, Year};

pub enum Msg {
    SelectPreviousYear,
    SelectPreviousYears,
    SelectYear(Year),
    SelectNextYear,
    SelectNextYears,
    SelectPreviousMonth,
    SelectMonth(Month),
    SelectNextMonth,
    SelectDay(Day),
    InitMonthSelection,
    DestroyMonthSelection,
    InitYearSelection,
    DestroyYearSelection,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub value: UtcDateTime,
    pub guide_mode: GuideMode,
    pub min: Option<UtcDateTime>,
    pub max: Option<UtcDateTime>,
    pub onchange: Callback<UtcDateTime>,
}

pub struct CrudDatetimeDateSelector {
    staging: UtcDateTime,

    short_week_day_names: Vec<String>,

    // TODO: Could use statically sized array type!
    years: Vec<Year>,
    months: Vec<Month>,
    weeks: Vec<Week>,

    show_year_selection: bool,
    show_month_selection: bool,
}

impl CrudDatetimeDateSelector {
    pub fn create_week_day_names(value: &UtcDateTime) -> Vec<String> {
        //let day_in_month = value.0.date().day(); // 1 based
        //let days_from_monday = value.0.date().weekday().num_days_from_monday(); // 0 based
        //let monday = value.0.date().with_day(day_in_month - days_from_monday).format("%a").to_string();

        vec![
            "Mon".to_owned(),
            "Tue".to_owned(),
            "Wed".to_owned(),
            "Thr".to_owned(),
            "Fri".to_owned(),
            "Sat".to_owned(),
            "Sun".to_owned(),
        ]
    }

    pub fn create_years(
        value: &UtcDateTime,
        starting_year: Option<i32>,
        min: Option<&UtcDateTime>,
        max: Option<&UtcDateTime>,
    ) -> Vec<Year> {
        let amount = 3 * 7; // 7 rows of 3 year numbers each.
        let starting_year = starting_year.unwrap_or_else(|| value.0.year() - 20);
        let mut years = Vec::<Year>::with_capacity(amount);
        let this_year = value.0.year();
        for i in 0..amount {
            let year_number = starting_year + i as i32;
            years.push(Year {
                number: year_number,
                is_now: year_number == this_year,
                disabled: !Self::is_in_range(
                    &value.0.with_year(year_number).unwrap(),
                    min.map(|it| &it.0),
                    max.map(|it| &it.0),
                ),
            });
        }
        years
    }

    pub fn create_months(
        value: &UtcDateTime,
        min: Option<&UtcDateTime>,
        max: Option<&UtcDateTime>,
    ) -> Vec<Month> {
        let now = UtcDateTime::now();
        let mut months = Vec::<Month>::with_capacity(12);
        for i in 1..=12 {
            let month = value.0.with_month(i).unwrap();
            months.push(Month {
                index: i,
                name: month.format("%B").to_string(),
                is_now: now.0.year() == month.year() && now.0.month() == month.month(),
                disabled: !Self::is_in_range(&month, max.map(|it| &it.0), min.map(|it| &it.0)),
            });
        }
        assert_eq!(months.len(), 12);
        months
    }

    pub fn is_in_range(
        date: &NaiveDateTime,
        min: Option<&NaiveDateTime>,
        max: Option<&NaiveDateTime>,
    ) -> bool {
        let after_min = match min {
            Some(min) => date >= min,
            None => true,
        };
        let before_max = match max {
            Some(max) => date <= max,
            None => true,
        };
        after_min && before_max
    }

    pub fn create_weeks(
        value: &UtcDateTime,
        min: Option<&UtcDateTime>,
        max: Option<&UtcDateTime>,
    ) -> Vec<Week> {
        let now = UtcDateTime::now();
        // Calculate the index of the first day of the month (in current locale).

        let current_year = now.0.year();
        let current_month = now.0.month();
        let current_day = now.0.day();
        let this_day = value.0.day();

        let first_weekday_index = value.first_day_of_month().num_days_from_monday(); // in range [0..6]
        let number_of_days_in_month = value.days_in_current_month();
        let index_of_last_day_in_month = first_weekday_index + number_of_days_in_month;

        let prev_month = value.previous_month();
        let next_month = value.next_month();

        let days_in_previous_month = prev_month.days_in_current_month();

        let current_day_lies_in_prev_month: bool = current_month == prev_month.0.month();
        let current_day_lies_in_this_month: bool = current_month == value.0.month();
        let current_day_lies_in_next_month: bool = current_month == next_month.0.month();

        let mut weeks = Vec::<Week>::with_capacity(6);
        for w in 0..6 {
            // 6 weeks to display.
            let mut week = Week {
                days: Vec::with_capacity(7),
            };
            for d in 0..7 {
                // 7 days each week.
                let i = d + w * 7;

                //let day = Day {
                //  index: -1,
                //  display_name: "".to_owned(),
                //  in_previous_month: false,
                //  in_current_month: false,
                //  in_next_month: false,
                //  disabled: false,
                //  highlighted: false,
                //  selected: false,
                //  is_now: false
                //};
                let day = if i < first_weekday_index {
                    let day_in_prev_month = days_in_previous_month - first_weekday_index + i + 1; // base 1 (!)
                    Day {
                        index: day_in_prev_month,
                        display_name: day_in_prev_month.to_string(),
                        in_previous_month: true,
                        in_current_month: false,
                        in_next_month: false,
                        utc_date_time: UtcDateTime(
                            prev_month.0.with_day(day_in_prev_month).unwrap(),
                        ),
                        disabled: !Self::is_in_range(
                            &prev_month.0.with_day(day_in_prev_month).unwrap(),
                            max.map(|it| &it.0),
                            min.map(|it| &it.0),
                        ),
                        highlighted: false,
                        // TODO: Can a day form prev not be selected?
                        selected: false,
                        is_now: match current_day_lies_in_prev_month {
                            // TODO: is year check necessary?
                            true => {
                                current_year == prev_month.0.year()
                                    && current_day == day_in_prev_month
                            }
                            false => false,
                        },
                    }
                } else if i >= first_weekday_index && i < index_of_last_day_in_month {
                    let day_in_month = i - first_weekday_index + 1; // base 1 (!)
                    Day {
                        index: day_in_month,
                        display_name: day_in_month.to_string(),
                        in_previous_month: false,
                        in_current_month: true,
                        in_next_month: false,
                        utc_date_time: UtcDateTime(value.0.with_day(day_in_month).unwrap()),
                        disabled: !Self::is_in_range(
                            &value.0.with_day(day_in_month).unwrap(),
                            max.map(|it| &it.0),
                            min.map(|it| &it.0),
                        ),
                        highlighted: false,
                        selected: this_day == day_in_month,
                        is_now: match current_day_lies_in_this_month {
                            // TODO: is year check necessary?
                            true => current_year == value.0.year() && current_day == day_in_month,
                            false => false,
                        },
                    }
                } else {
                    let day_in_next_month = i - index_of_last_day_in_month + 1; // base 1 (!)
                    Day {
                        index: day_in_next_month,
                        display_name: day_in_next_month.to_string(),
                        in_previous_month: false,
                        in_current_month: false,
                        in_next_month: true,
                        utc_date_time: UtcDateTime(
                            next_month.0.with_day(day_in_next_month).unwrap(),
                        ),
                        disabled: !Self::is_in_range(
                            &next_month.0.with_day(day_in_next_month).unwrap(),
                            max.map(|it| &it.0),
                            min.map(|it| &it.0),
                        ),
                        highlighted: false,
                        // TODO: not selectable?
                        selected: false,
                        is_now: match current_day_lies_in_next_month {
                            // TODO: is year check necessary?
                            true => {
                                current_year == next_month.0.year()
                                    && current_day == day_in_next_month
                            }
                            false => false,
                        },
                    }
                };
                week.days.push(day);
            }
            weeks.push(week);
        }
        weeks
    }

    pub fn format_years_range(&self) -> String {
        if self.years.is_empty() {
            "ERR: no years".to_owned()
        } else {
            format!(
                "{} - {}",
                self.years[0].number,
                self.years[self.years.len() - 1].number
            )
        }
    }
}

impl Component for CrudDatetimeDateSelector {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            staging: ctx.props().value.clone(),
            short_week_day_names: Self::create_week_day_names(&ctx.props().value),
            years: Self::create_years(
                &ctx.props().value,
                None,
                ctx.props().min.as_ref(),
                ctx.props().max.as_ref(),
            ),
            months: Self::create_months(
                &ctx.props().value,
                ctx.props().min.as_ref(),
                ctx.props().max.as_ref(),
            ),
            weeks: Self::create_weeks(
                &ctx.props().value,
                ctx.props().min.as_ref(),
                ctx.props().max.as_ref(),
            ),
            show_year_selection: false,
            show_month_selection: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SelectPreviousYear => {
                self.staging = UtcDateTime(self.staging.0.with_year(self.staging.0.year() - 1).unwrap());
                self.years = Self::create_years(&self.staging, Some(self.staging.0.year() - 4),  ctx.props().min.as_ref(), ctx.props().max.as_ref());
                self.months = Self::create_months(&ctx.props().value, ctx.props().min.as_ref(), ctx.props().max.as_ref());
                ctx.props().onchange.emit(self.staging.clone());
                true
            },
            Msg::SelectPreviousYears => {
                let starting_at = match self.years.len() {
                    0 => None,
                    _ => Some(self.years[0].number - (3 * 7)),
                };
                self.years = Self::create_years(
                    &self.staging,
                    starting_at,
                    ctx.props().min.as_ref(),
                    ctx.props().max.as_ref(),
                );
                true
            }
            Msg::SelectYear(year) => {
                if year.disabled {
                    return false;
                }
                ctx.props().onchange.emit(UtcDateTime(self.staging.0.with_year(year.number).unwrap()));
                ctx.link().send_message(Msg::InitMonthSelection);
                ctx.link().send_message(Msg::DestroyYearSelection);
                true
            }
            Msg::SelectNextYear => {
                self.staging = UtcDateTime(self.staging.0.with_year(self.staging.0.year() + 1).unwrap());
                self.years = Self::create_years(&self.staging, Some(self.staging.0.year() - 4),  ctx.props().min.as_ref(), ctx.props().max.as_ref());
                self.months = Self::create_months(&ctx.props().value, ctx.props().min.as_ref(), ctx.props().max.as_ref());
                ctx.props().onchange.emit(self.staging.clone());
                true
            },
            Msg::SelectNextYears => {
                let starting_at = match self.years.len() {
                    0 => None,
                    _ => Some(self.years[self.years.len() - 1].number + 1),
                };
                self.years = Self::create_years(
                    &self.staging,
                    starting_at,
                    ctx.props().min.as_ref(),
                    ctx.props().max.as_ref(),
                );
                true
            }
            Msg::SelectPreviousMonth => {
                self.staging = self.staging.previous_month();
                self.years = Self::create_years(&self.staging, None,  ctx.props().min.as_ref(), ctx.props().max.as_ref());
                self.months = Self::create_months(&ctx.props().value, ctx.props().min.as_ref(), ctx.props().max.as_ref());
                self.weeks = Self::create_weeks(&self.staging, ctx.props().min.as_ref(), ctx.props().max.as_ref());
                ctx.props().onchange.emit(self.staging.clone());
                true
            },
            Msg::SelectMonth(month) => {
                if month.disabled {
                    return false;
                }
                self.staging = UtcDateTime(self.staging.0.with_month(month.index).unwrap());
                ctx.link().send_message(Msg::DestroyMonthSelection);
                if ctx.props().guide_mode == GuideMode::YearFirst {
                    self.months = Self::create_months(&ctx.props().value, ctx.props().min.as_ref(), ctx.props().max.as_ref());
                }
                self.weeks = Self::create_weeks(&self.staging, ctx.props().min.as_ref(), ctx.props().max.as_ref());
                true
            },
            Msg::SelectNextMonth => {
                self.staging = self.staging.next_month();
                self.years = Self::create_years(&self.staging, None,  ctx.props().min.as_ref(), ctx.props().max.as_ref());
                self.months = Self::create_months(&ctx.props().value, ctx.props().min.as_ref(), ctx.props().max.as_ref());
                self.weeks = Self::create_weeks(&self.staging, ctx.props().min.as_ref(), ctx.props().max.as_ref());
                ctx.props().onchange.emit(self.staging.clone());
                true
            },
            Msg::SelectDay(day) => {
                self.staging = day.utc_date_time;
                self.weeks = Self::create_weeks(&self.staging, ctx.props().min.as_ref(), ctx.props().max.as_ref());
                ctx.props().onchange.emit(self.staging.clone());
                true
            }
            Msg::InitMonthSelection => {
                self.months = Self::create_months(&ctx.props().value, ctx.props().min.as_ref(), ctx.props().max.as_ref());
                self.show_month_selection = true;
                true
            }
            Msg::DestroyMonthSelection => {
                self.show_month_selection = false;
                true
            }
            Msg::InitYearSelection => {
                ctx.link().send_message(Msg::DestroyMonthSelection);
                self.years = Self::create_years(&self.staging, None,  ctx.props().min.as_ref(), ctx.props().max.as_ref());
                self.show_year_selection = true;
                true
            }
            Msg::DestroyYearSelection => {
                self.show_year_selection = false;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class={"date-selector"}>
                <div class={"datetime-calendar-month"}>

                    <div class={"actions"}>

                        if !self.show_year_selection && !self.show_month_selection {
                            <div onclick={ctx.link().callback(|_| Msg::SelectPreviousMonth)}
                                 class={"previous arrow-left"}>
                            </div>
                        }

                        if self.show_month_selection {
                            <div onclick={ctx.link().callback(|_| Msg::SelectPreviousYear)}
                                 class={"previous arrow-left"}>
                            </div>
                        }

                        if self.show_year_selection {
                            <div onclick={ctx.link().callback(|_| Msg::SelectPreviousYears)}
                                 class={"previous arrow-left"}>
                            </div>
                        }

                        if self.show_year_selection {
                            <div onclick={ctx.link().callback(|_| Msg::DestroyYearSelection)}
                                 class={"current-date"}>
                                {self.format_years_range()}
                            </div>
                        }

                        if self.show_month_selection {
                            <div onclick={ctx.link().callback(|_| Msg::InitYearSelection)}
                                 class={"current-date"}>
                                {self.staging.0.year()}
                            </div>
                        }

                        if !self.show_month_selection && !self.show_year_selection {
                            <div onclick={ctx.link().callback(|_| Msg::InitYearSelection)}
                                 class={"current-date"}>
                                {self.staging.0.month()}
                                {self.staging.0.year()}
                            </div>
                        }

                        if !self.show_month_selection && !self.show_year_selection {
                            <div onclick={ctx.link().callback(|_| Msg::SelectNextMonth)}
                                 class={"next arrow-right"}>
                            </div>
                        }

                        if self.show_month_selection {
                            <div onclick={ctx.link().callback(|_| Msg::SelectNextYear)}
                                 class={"next arrow-right"}>
                            </div>
                        }


                        if self.show_year_selection {
                            <div onclick={ctx.link().callback(|_| Msg::SelectNextYears)}
                                 class={"next arrow-right"}>
                            </div>
                        }
                    </div>

                    if self.show_year_selection {
                        <div class={"years"}>
                            {
                                self.years.iter().map(|year| {
                                    let year_callback = year.clone();
                                    html!{
                                        <div onclick={ctx.link().callback(move |_| Msg::SelectYear(year_callback.clone()))}
                                            class={classes!("year", year.is_now.then(|| "is-now"), year.disabled.then(|| "disabled"))}>
                                            {year.number.to_string()}
                                        </div>
                                    }
                                }).collect::<Html>()
                            }
                        </div>
                    }

                    if self.show_month_selection {
                        <div class={"months"}>
                            {
                                self.months.iter().map(|month| {
                                    let month_callback = month.clone();
                                     html!{
                                        <div onclick={ctx.link().callback(move |_| Msg::SelectMonth(month_callback.clone()))}
                                            class={classes!("month", month.is_now.then(|| "is-now"), month.disabled.then(|| "disabled"))}>
                                            {month.name.clone()}
                                        </div>
                                    }
                                }).collect::<Html>()
                            }
                        </div>
                    }

                   if !self.show_year_selection && !self.show_month_selection {
                        <div class={"weekday-names"}>
                            {
                                self.short_week_day_names.iter().map(|day_name| html!{
                                    <div class={"weekday-name"}>
                                        {day_name.clone()}
                                    </div>
                                }).collect::<Html>()
                            }
                        </div>

                        {
                            self.weeks.iter().map(|week| {
                                let week = week.clone();
                                html!{
                                    <div class={"week"}>
                                        {
                                            week.days.iter().map(|day| {
                                                let day_callback = day.clone();
                                                html!{
                                                    <div
                                                        onclick={ctx.link().callback(move |_| Msg::SelectDay(day_callback.clone()))}
                                                        class={classes!("day", (!day.in_current_month).then(|| "not-in-month"), day.disabled.then(|| "disabled"), day.selected.then(|| "selected"))}>
                                                        <span class={classes!("text", day.is_now.then(|| "is-now"))}>
                                                            {day.display_name.clone()}
                                                        </span>
                                                    </div>
                                                }
                                            }).collect::<Html>()
                                        }
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    }
                </div>
            </div>
        }
    }
}
