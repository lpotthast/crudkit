use std::{borrow::Cow, fmt::Display};

use leptonic::prelude::*;
use leptos::*;

#[component]
pub fn CrudPagination(
    cx: Scope,
    #[prop(into)] item_count: MaybeSignal<u64>,
    #[prop(into)] current_page: Signal<u64>,
    set_current_page: Callback<u64>,
    #[prop(into, default = 5.into())] items_per_page: MaybeSignal<u64>,
    set_items_per_page: Callback<u64>,
) -> impl IntoView {
    let page_count = Signal::derive(cx, move || {
        (item_count.get() as f64 / items_per_page.get() as f64).ceil() as u64
    });

    let page_options = Signal::derive(cx, move || {
        create_page_options(page_count.get(), current_page.get())
    });

    let items_per_page_options = Signal::derive(cx, move || {
        let items_per_page = items_per_page.get();

        let mut default_options = vec![
            ItemsPerPage::some(10),
            ItemsPerPage::some(25),
            ItemsPerPage::some(50),
            ItemsPerPage::some(100),
        ];

        // Note: The set_items_per_page must always be in the returned vec, as this is always default-selected in our dropdown!
        if default_options
            .iter()
            .find(|it| it.items_per_page == items_per_page)
            .is_none()
        {
            default_options.push(ItemsPerPage::some(items_per_page));
        }

        default_options.sort_by(|a, b| a.items_per_page.cmp(&b.items_per_page));
        default_options
    });

    let set_items_per_page = Callback::new(cx, move |option: ItemsPerPage| {
        set_items_per_page.call(option.items_per_page);

        // We may have to update the current page as well if it would not show any element anymore!
        let new_page_count = (item_count.get() as f64 / option.items_per_page as f64).ceil() as u64;
        if current_page.get_untracked() > new_page_count {
            set_current_page.call(new_page_count);
        }
    });

    view! {cx,
        <Show when=move || { item_count.get() > 0 } fallback=|_| ()>
            <Grid spacing=6 class="crud-pagination">
                <Row>
                    <Col h_align=ColAlign::Start>
                        <div class="items-per-page-selector">
                            <div class="label">
                                "Einträge pro Seite"
                            </div>
                            <Select
                                options=items_per_page_options
                                render_option=Callback::new(cx, move |(cx, option)| format!("{option}").into_view(cx))
                                selected=Signal::derive(cx, move || ItemsPerPage::some(items_per_page.get()))
                                set_selected=set_items_per_page
                            />
                        </div>
                    </Col>

                    <Col h_align=ColAlign::End>
                        <ButtonGroup>
                            {
                                move || {
                                    let page_options = page_options.get();
                                    page_options.options.into_iter().map(|page_number| {
                                        view! {cx,
                                            <Button
                                                variant=ButtonVariant::Filled
                                                color=ButtonColor::Secondary
                                                disabled=page_number.is_none()
                                                active=MaybeSignal::from(page_number == Some(page_options.for_current_page)) // TODO: Use signal::derive instead?
                                                on_click=move |_| if let Some(number) = page_number {
                                                    set_current_page.call(number)
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
                            }
                        </ButtonGroup>
                    </Col>
                </Row>
            </Grid>
        </Show>
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct ItemsPerPage {
    items_per_page: u64,
    all: bool,
}

impl ItemsPerPage {
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

impl Display for ItemsPerPage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.all {
            true => f.write_fmt(format_args!("All ({})", self.items_per_page)),
            false => f.write_fmt(format_args!("{}", self.items_per_page)),
        }
    }
}

#[derive(Debug, Clone)]
struct PageOptions {
    options: Vec<Option<u64>>,
    for_current_page: u64,
}

fn create_page_options(page_count: u64, current_page: u64) -> PageOptions {
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
    PageOptions {
        options,
        for_current_page: current_page,
    }
}
