use std::fmt::Display;

use yew::prelude::*;

use crate::crud_select::{OptionRenderer, Selection};
use crate::prelude::CrudSelect;

use super::crud_btn::CrudBtn;
use super::crud_btn_group::CrudBtnGroup;
use super::Variant;

pub enum Msg {
    PageSelected(u64),
    ItemCountSelected(Selection<ItemsPerPageOption>),
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub current_page: u64,
    pub item_count: u64,
    #[prop_or(5)]
    pub items_per_page: u64,
    pub on_page_select: Callback<u64>,
    pub on_item_count_select: Callback<u64>,
}

pub struct CrudPagination {
    page_count: u64,
    page_options: Vec<Option<u64>>,
    items_per_page_options: Vec<ItemsPerPageOption>,
}

impl Component for CrudPagination {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let page_count = compute_page_count(ctx.props().item_count, ctx.props().items_per_page);
        let page_options = compute_page_options(page_count, ctx.props().current_page);
        let items_per_page_options = compute_items_per_page_options(ctx.props().items_per_page);
        Self {
            page_count,
            page_options,
            items_per_page_options,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::PageSelected(page_number) => {
                ctx.props().on_page_select.emit(page_number);
                false
            }
            Msg::ItemCountSelected(selection) => {
                match selection {
                    Selection::None => panic!("selecting `Selection::None` should not be possible"),
                    Selection::Single(option) => {
                        ctx.props().on_item_count_select.emit(option.items_per_page)
                    }
                    Selection::Multiple(_) => {
                        panic!("selecting `Selection::Multiple(_)` should not be possible")
                    }
                }
                false
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.page_count = compute_page_count(ctx.props().item_count, ctx.props().items_per_page);
        self.page_options = compute_page_options(self.page_count, ctx.props().current_page);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match ctx.props().item_count > 0 {
            true => html! {
                <div class={"crud-row crud-pagination"}>
                    <div class={"crud-col crud-col-flex crud-col-flex-start"}>
                    </div>

                    <div class={"crud-col crud-col-flex crud-col-flex-row crud-col-flex-end"}>
                        <div class={"items-per-page-selector"}>
                            <div class={"label"}>
                                {"Einträge pro Seite"}
                            </div>
                            <CrudSelect<ItemsPerPageOption>
                                options={self.items_per_page_options.clone()}
                                option_renderer={Some(OptionRenderer {
                                    name: "default",
                                    renderer: |it: &ItemsPerPageOption| html! { format!("{}", it.items_per_page) },
                                })}
                                selected={Selection::Single(ItemsPerPageOption::some(ctx.props().items_per_page))}
                                selection_changed={ctx.link().callback(Msg::ItemCountSelected)}>
                            </CrudSelect<ItemsPerPageOption>>
                        </div>
                        <CrudBtnGroup>
                            {
                                self.page_options.iter().map(|page_number| {
                                    let name = match page_number {
                                        Some(page_number) => format!("{}", page_number),
                                        None => '\u{2026}'.to_string(), // `hellip` character
                                    };
                                    html! {
                                        <CrudBtn
                                            name={ name }
                                            variant={ Variant::Default }
                                            disabled={ page_number.is_none() }
                                            active={ *page_number == Some(ctx.props().current_page) }
                                            onclick={
                                                match *page_number {
                                                    Some(number) => ctx.link().batch_callback(move |_| Some(Msg::PageSelected(number))),
                                                    None => ctx.link().batch_callback(|_| None),
                                                }
                                            }
                                        />
                                    }
                                }).collect::<Html>()
                            }
                        </CrudBtnGroup>
                    </div>
                </div>
            },
            false => html! {},
        }
    }
}

fn compute_page_count(item_count: u64, items_per_page: u64) -> u64 {
    (item_count as f64 / items_per_page as f64).ceil() as u64
}

fn compute_page_options(page_count: u64, current_page: u64) -> Vec<Option<u64>> {
    let mut options: Vec<Option<u64>>;
    // Just return all available pages if there are not too many of them.
    if page_count <= 10 {
        options = Vec::with_capacity(page_count as usize);
        for i in 1..=page_count {
            options.push(Some(i));
        }
    }
    // Single ... at the right. Start of page spectrum.
    else if current_page <= 5 {
        options = vec![
            Some(1),
            Some(2),
            Some(3),
            Some(4),
            Some(5),
            Some(6),
            Some(7),
            None,
            Some(page_count - 1),
            Some(page_count),
        ];
    }
    // With ... at the left and right. In the middle of the available pages.
    else if current_page > 5 && current_page < page_count - 4 {
        options = vec![
            Some(1),
            Some(2),
            None,
            Some(current_page - 2),
            Some(current_page - 1),
            Some(current_page),
            Some(current_page + 1),
            Some(current_page + 2),
            None,
            Some(page_count - 1),
            Some(page_count),
        ];
    }
    // Single ... at the left. End of page spectrum.
    else if current_page >= page_count - 4 {
        options = vec![
            Some(1),
            Some(2),
            None,
            Some(page_count - 6),
            Some(page_count - 5),
            Some(page_count - 4),
            Some(page_count - 3),
            Some(page_count - 2),
            Some(page_count - 1),
            Some(page_count),
        ];
    }
    // Error...
    else {
        panic!("Unreachable!");
    }
    options
}

fn compute_items_per_page_options(set_items_per_page: u64) -> Vec<ItemsPerPageOption> {
    let mut default_options = vec![
        ItemsPerPageOption::some(10),
        ItemsPerPageOption::some(25),
        ItemsPerPageOption::some(50),
        ItemsPerPageOption::some(100),
    ];

    // Note: The set_items_per_page must always be in the returned vec, as this is always default-selected in our dropdown!
    if default_options
        .iter()
        .find(|it| it.items_per_page == set_items_per_page)
        .is_none()
    {
        default_options.push(ItemsPerPageOption::some(set_items_per_page));
    }

    default_options.sort_by(|a, b| a.items_per_page.cmp(&b.items_per_page));
    default_options
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ItemsPerPageOption {
    items_per_page: u64,
    all: bool,
}

impl ItemsPerPageOption {
    pub fn some(items_per_page: u64) -> Self {
        Self {
            items_per_page,
            all: false,
        }
    }

    pub fn all(items_per_page: u64) -> Self {
        Self {
            items_per_page,
            all: true,
        }
    }
}

impl Display for ItemsPerPageOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.all {
            true => f.write_fmt(format_args!("All ({})", self.items_per_page)),
            false => f.write_fmt(format_args!("{}", self.items_per_page)),
        }
    }
}
