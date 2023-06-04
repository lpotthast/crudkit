use std::{borrow::Cow, fmt::Display};

use leptonic::prelude::*;
use leptos::*;

#[component]
pub fn CrudPagination(
    cx: Scope,
    #[prop(into)] current_page: Signal<u64>,
    #[prop(into)] item_count: MaybeSignal<u64>,
    #[prop(into, default = 5.into())] items_per_page: MaybeSignal<u64>,
    on_page_select: Callback<u64>,
    on_item_count_select: Callback<u64>,
) -> impl IntoView {
    let page_count = Signal::derive(cx, move || {
        (item_count.get() as f64 / items_per_page.get() as f64).ceil() as u64
    });

    let page_options = Signal::derive(cx, move || {
        let page_count = page_count.get();
        let current_page = current_page.get();

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
    });

    let items_per_page_options = Signal::derive(cx, move || {
        let items_per_page = items_per_page.get();

        let mut default_options = vec![
            ItemsPerPageOption::some(10),
            ItemsPerPageOption::some(25),
            ItemsPerPageOption::some(50),
            ItemsPerPageOption::some(100),
        ];

        // Note: The set_items_per_page must always be in the returned vec, as this is always default-selected in our dropdown!
        if default_options
            .iter()
            .find(|it| it.items_per_page == items_per_page)
            .is_none()
        {
            default_options.push(ItemsPerPageOption::some(items_per_page));
        }

        default_options.sort_by(|a, b| a.items_per_page.cmp(&b.items_per_page));
        default_options
    });

    move || {
        let page_options = page_options.get();
        let item_count = item_count.get();

        (item_count > 0).then(|| view! {cx,
        <Grid spacing=6 class="crud-pagination">
            <Row>
                <Col h_align=ColAlign::Start> // crud-col-flex
                    <div class="items-per-page-selector">
                        <div class="label">
                            "Einträge pro Seite"
                        </div>
                        <Select
                            mode=SelectMode::Single
                            options=items_per_page_options
                            render_option=move |cx, option| view! {cx,
                                { format!("{}", option) }
                            }
                            // TODO: Pass selected!
                            //selected={Selection::Single(ItemsPerPageOption::some(ctx.props().items_per_page))}
                            //selection_changed={ctx.link().callback(Msg::ItemCountSelected)}
                            >
                        </Select>
                    </div>
                </Col>

                <Col h_align=ColAlign::End> // crud-col-flex crud-col-flex-row
                    <ButtonGroup>
                        {
                            page_options.into_iter().map(|page_number| {
                                view! {cx,
                                    <Button
                                        variant=ButtonVariant::Filled
                                        color=ButtonColor::Secondary
                                        disabled=page_number.is_none()
                                        active=MaybeSignal::from(page_number == Some(current_page.get()))
                                        on_click=move |_| if let Some(number) = page_number {
                                            on_page_select.call(number)
                                        }
                                    >
                                        { match page_number {
                                            Some(page_number) => Cow::Owned(format!("{}", page_number)),
                                            None => Cow::Borrowed("\u{2026}"), // The `hellip` character (three dots)
                                        } }
                                    </Button>
                                }
                            }).collect_view(cx)
                        }
                    </ButtonGroup>
                </Col>
            </Row>
        </Grid>
    })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
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
