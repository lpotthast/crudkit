use yew::prelude::*;

use crate::crud_offset_datetime::{Day, GuideMode, Month, Week, Year};

use time::macros::format_description;

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
    pub value: time::OffsetDateTime,
    pub guide_mode: GuideMode,
    pub min: Option<time::OffsetDateTime>,
    pub max: Option<time::OffsetDateTime>,
    pub onchange: Callback<time::OffsetDateTime>,
}

pub struct CrudOffsetDatetimeDateSelector {
    staging: time::OffsetDateTime,

    short_week_day_names: Vec<String>,

    // TODO: Could use statically sized array type!
    years: Vec<Year>,
    months: Vec<Month>,
    weeks: Vec<Week>,

    show_year_selection: bool,
    show_month_selection: bool,
}

impl CrudOffsetDatetimeDateSelector {
    pub fn create_week_day_names(value: &time::OffsetDateTime) -> Vec<String> {
        //let day_in_month = value.date().day(); // 1 based
        //let days_from_monday = value.date().weekday().num_days_from_monday(); // 0 based
        //let monday = value.date().with_day(day_in_month - days_from_monday).format("%a").to_string();

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
        value: &time::OffsetDateTime,
        starting_year: Option<i32>,
        min: Option<&time::OffsetDateTime>,
        max: Option<&time::OffsetDateTime>,
    ) -> Vec<Year> {
        let amount = 3 * 7; // 7 rows of 3 year numbers each.
        let starting_year = starting_year.unwrap_or_else(|| value.year() - 20);
        let mut years = Vec::<Year>::with_capacity(amount);
        let this_year = value.year();
        for i in 0..amount {
            let year_number = starting_year + i as i32;
            years.push(Year {
                number: year_number,
                is_now: year_number == this_year,
                disabled: !Self::is_in_range(&value.replace_year(year_number).unwrap(), min, max),
            });
        }
        years
    }

    pub fn create_months(
        value: &time::OffsetDateTime,
        min: Option<&time::OffsetDateTime>,
        max: Option<&time::OffsetDateTime>,
    ) -> Vec<Month> {
        let now = time::OffsetDateTime::now_utc();
        let mut months = Vec::<Month>::with_capacity(12);
        for i in 1..=12u8 {
            let month = value
                .replace_month(time::Month::try_from(i).unwrap())
                .unwrap();
            months.push(Month {
                index: i,
                name: month.format(format_description!("[month]")).unwrap(),
                is_now: now.year() == month.year() && now.month() == month.month(),
                disabled: !Self::is_in_range(&month, max, min),
            });
        }
        assert_eq!(months.len(), 12);
        months
    }

    pub fn is_in_range(
        date: &time::OffsetDateTime,
        min: Option<&time::OffsetDateTime>,
        max: Option<&time::OffsetDateTime>,
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

    pub fn whole_days_in(year: i32, month: time::Month) -> u8 {
        let duration = time::Date::from_calendar_date(
            match month {
                time::Month::December => year + 1,
                _ => year,
            },
            month.next(),
            1,
        )
        .unwrap()
            - time::Date::from_calendar_date(year, month, 1).unwrap();
        duration.whole_days() as u8
    }

    /// Might decrease the year to x-1 if in January of year x.
    pub fn start_of_previous_month(dt: time::OffsetDateTime) -> time::OffsetDateTime {
        let start = dt.replace_day(1).unwrap();
        match start.month() {
            time::Month::January => start
                .replace_year(start.year() - 1)
                .unwrap()
                .replace_month(time::Month::December)
                .unwrap(),
            _ => start.replace_month(start.month().previous()).unwrap(),
        }
    }

    /// Might advance the year to x+1 if in December of year x.
    pub fn start_of_next_month(dt: time::OffsetDateTime) -> time::OffsetDateTime {
        let start = dt.replace_day(1).unwrap();
        match start.month() {
            time::Month::December => start
                .replace_year(start.year() + 1)
                .unwrap()
                .replace_month(time::Month::January)
                .unwrap(),
            _ => start.replace_month(start.month().next()).unwrap(),
        }
    }

    pub fn create_weeks(
        value: &time::OffsetDateTime,
        min: Option<&time::OffsetDateTime>,
        max: Option<&time::OffsetDateTime>,
    ) -> Vec<Week> {
        let now = time::OffsetDateTime::now_utc();
        // Calculate the index of the first day of the month (in current locale).

        let current_year = now.year();
        let current_month = now.month();
        let current_day = now.day();
        let this_day = value.day();

        let first_weekday_index = value
            .clone()
            .replace_day(1)
            .unwrap()
            .weekday()
            .number_days_from_monday(); // in range [0..6]
        let number_of_days_in_month = Self::whole_days_in(value.year(), value.month());
        let index_of_last_day_in_month = first_weekday_index + number_of_days_in_month;

        let prev_month = Self::start_of_previous_month(value.clone());
        let next_month = Self::start_of_next_month(value.clone());

        let days_in_previous_month = Self::whole_days_in(prev_month.year(), prev_month.month());

        let current_day_lies_in_prev_month: bool = current_month == prev_month.month();
        let current_day_lies_in_this_month: bool = current_month == value.month();
        let current_day_lies_in_next_month: bool = current_month == next_month.month();

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
                    let date_time = value.replace_day(day_in_prev_month).unwrap();
                    let disabled = !Self::is_in_range(&date_time, max, min); // TODO: inversion of min/max correct?
                    Day {
                        index: day_in_prev_month,
                        display_name: day_in_prev_month.to_string(),
                        in_previous_month: true,
                        in_current_month: false,
                        in_next_month: false,
                        date_time,
                        disabled,
                        highlighted: false,
                        // TODO: Can a day form prev not be selected?
                        selected: false,
                        is_now: match current_day_lies_in_prev_month {
                            // TODO: is year check necessary?
                            true => {
                                current_year == prev_month.year()
                                    && current_day == day_in_prev_month
                            }
                            false => false,
                        },
                    }
                } else if i >= first_weekday_index && i < index_of_last_day_in_month {
                    let day_in_month: u8 = i - first_weekday_index + 1; // base 1 (!)
                    let date_time = value.replace_day(day_in_month).unwrap();
                    let disabled = !Self::is_in_range(&date_time, max, min); // TODO: inversion of min/max correct?
                    Day {
                        index: day_in_month,
                        display_name: day_in_month.to_string(),
                        in_previous_month: false,
                        in_current_month: true,
                        in_next_month: false,
                        date_time,
                        disabled,
                        highlighted: false,
                        selected: this_day == day_in_month,
                        is_now: match current_day_lies_in_this_month {
                            // TODO: is year check necessary?
                            true => current_year == value.year() && current_day == day_in_month,
                            false => false,
                        },
                    }
                } else {
                    let day_in_next_month: u8 = i - index_of_last_day_in_month + 1; // base 1 (!)
                    let date_time = value.replace_day(day_in_next_month).unwrap();
                    let disabled = !Self::is_in_range(&date_time, max, min); // TODO: inversion of min/max correct?
                    Day {
                        index: day_in_next_month,
                        display_name: day_in_next_month.to_string(),
                        in_previous_month: false,
                        in_current_month: false,
                        in_next_month: true,
                        date_time,
                        disabled,
                        highlighted: false,
                        // TODO: not selectable?
                        selected: false,
                        is_now: match current_day_lies_in_next_month {
                            // TODO: is year check necessary?
                            true => {
                                current_year == next_month.year()
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

impl Component for CrudOffsetDatetimeDateSelector {
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
                self.staging = self.staging.replace_year(self.staging.year() - 1).unwrap();
                self.years = Self::create_years(
                    &self.staging,
                    Some(self.staging.year() - 4),
                    ctx.props().min.as_ref(),
                    ctx.props().max.as_ref(),
                );
                self.months = Self::create_months(
                    &ctx.props().value,
                    ctx.props().min.as_ref(),
                    ctx.props().max.as_ref(),
                );
                ctx.props().onchange.emit(self.staging.clone());
                true
            }
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
                ctx.props()
                    .onchange
                    .emit(self.staging.replace_year(year.number).unwrap());
                ctx.link().send_message(Msg::InitMonthSelection);
                ctx.link().send_message(Msg::DestroyYearSelection);
                true
            }
            Msg::SelectNextYear => {
                self.staging = self.staging.replace_year(self.staging.year() + 1).unwrap();
                self.years = Self::create_years(
                    &self.staging,
                    Some(self.staging.year() - 4),
                    ctx.props().min.as_ref(),
                    ctx.props().max.as_ref(),
                );
                self.months = Self::create_months(
                    &ctx.props().value,
                    ctx.props().min.as_ref(),
                    ctx.props().max.as_ref(),
                );
                ctx.props().onchange.emit(self.staging.clone());
                true
            }
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
                self.staging = Self::start_of_previous_month(self.staging);
                self.years = Self::create_years(
                    &self.staging,
                    None,
                    ctx.props().min.as_ref(),
                    ctx.props().max.as_ref(),
                );
                self.months = Self::create_months(
                    &ctx.props().value,
                    ctx.props().min.as_ref(),
                    ctx.props().max.as_ref(),
                );
                self.weeks = Self::create_weeks(
                    &self.staging,
                    ctx.props().min.as_ref(),
                    ctx.props().max.as_ref(),
                );
                ctx.props().onchange.emit(self.staging.clone());
                true
            }
            Msg::SelectMonth(month) => {
                if month.disabled {
                    return false;
                }
                self.staging = self
                    .staging
                    .replace_month(time::Month::try_from(month.index).unwrap())
                    .unwrap();
                ctx.link().send_message(Msg::DestroyMonthSelection);
                if ctx.props().guide_mode == GuideMode::YearFirst {
                    self.months = Self::create_months(
                        &ctx.props().value,
                        ctx.props().min.as_ref(),
                        ctx.props().max.as_ref(),
                    );
                }
                self.weeks = Self::create_weeks(
                    &self.staging,
                    ctx.props().min.as_ref(),
                    ctx.props().max.as_ref(),
                );
                true
            }
            Msg::SelectNextMonth => {
                self.staging = Self::start_of_next_month(self.staging);
                self.years = Self::create_years(
                    &self.staging,
                    None,
                    ctx.props().min.as_ref(),
                    ctx.props().max.as_ref(),
                );
                self.months = Self::create_months(
                    &ctx.props().value,
                    ctx.props().min.as_ref(),
                    ctx.props().max.as_ref(),
                );
                self.weeks = Self::create_weeks(
                    &self.staging,
                    ctx.props().min.as_ref(),
                    ctx.props().max.as_ref(),
                );
                ctx.props().onchange.emit(self.staging.clone());
                true
            }
            Msg::SelectDay(day) => {
                self.staging = day.date_time;
                self.weeks = Self::create_weeks(
                    &self.staging,
                    ctx.props().min.as_ref(),
                    ctx.props().max.as_ref(),
                );
                ctx.props().onchange.emit(self.staging.clone());
                true
            }
            Msg::InitMonthSelection => {
                self.months = Self::create_months(
                    &ctx.props().value,
                    ctx.props().min.as_ref(),
                    ctx.props().max.as_ref(),
                );
                self.show_month_selection = true;
                true
            }
            Msg::DestroyMonthSelection => {
                self.show_month_selection = false;
                true
            }
            Msg::InitYearSelection => {
                ctx.link().send_message(Msg::DestroyMonthSelection);
                self.years = Self::create_years(
                    &self.staging,
                    None,
                    ctx.props().min.as_ref(),
                    ctx.props().max.as_ref(),
                );
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
                                {self.staging.year()}
                            </div>
                        }

                        if !self.show_month_selection && !self.show_year_selection {
                            <div onclick={ctx.link().callback(|_| Msg::InitYearSelection)}
                                 class={"current-date"}>
                                {self.staging.month()}
                                {self.staging.year()}
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
