use crate::crud_instance_config::{ItemsPerPage, PageNr};
use leptonic::components::prelude::*;
use leptonic::prelude::*;
use leptos::prelude::*;
use std::cmp::PartialOrd;
use std::{borrow::Cow, fmt::Display};

#[component]
pub fn CrudPagination(
    #[prop(into)] item_count: Signal<u64>,
    #[prop(into)] current_page: Signal<PageNr>,
    #[prop(into)] set_current_page: Callback<PageNr>,
    #[prop(into)] items_per_page: Signal<ItemsPerPage>,
    #[prop(into)] set_items_per_page: Callback<ItemsPerPage>,
) -> impl IntoView {
    let page_count = Signal::derive(move || {
        (item_count.get() as f64 / items_per_page.get().0 as f64).ceil() as u64
    });

    let page_options =
        Signal::derive(move || PageOptions::new(page_count.get(), current_page.get().0));

    let items_per_page_options = Signal::derive(move || {
        let items_per_page = items_per_page.get();

        let mut default_options = vec![
            ItemsPerPageEntry::some(ItemsPerPage(10)),
            ItemsPerPageEntry::some(ItemsPerPage(25)),
            ItemsPerPageEntry::some(ItemsPerPage(50)),
            ItemsPerPageEntry::some(ItemsPerPage(100)),
        ];

        // Note: The set_items_per_page must always be in the returned vec, as this is always default-selected in our dropdown!
        if !default_options
            .iter().any(|it| it.items_per_page == items_per_page)
        {
            default_options.push(ItemsPerPageEntry::some(items_per_page));
        }

        default_options.sort_by(|a, b| a.items_per_page.cmp(&b.items_per_page));
        default_options
    });

    let set_items_per_page = Callback::new(move |option: ItemsPerPageEntry| {
        set_items_per_page.run(option.items_per_page);

        // We may have to update the current page as well if it would not show any element anymore!
        let new_page_nr =
            PageNr((item_count.get() as f64 / option.items_per_page.0 as f64).ceil() as u64);
        if current_page.get_untracked() > new_page_nr {
            set_current_page.run(new_page_nr);
        }
    });

    view! {
        <Show when=move || { item_count.get() > 0 } fallback=|| ()>
            <Grid gap=Size::Em(0.6) attr:class="crud-pagination">
                <Row>
                    <Col xs=6 h_align=ColAlign::Start>
                        <div class="items-per-page-selector">
                            <div class="label">"Einträge pro Seite"</div>
                            <Select
                                options=items_per_page_options
                                search_text_provider=move |o: ItemsPerPageEntry| { o.to_string() }
                                render_option=move |o: ItemsPerPageEntry| o.to_string().into_any()
                                selected=Signal::derive(move || ItemsPerPageEntry::some(items_per_page.get()))
                                set_selected=set_items_per_page
                            />
                        </div>
                    </Col>

                    <Col xs=6 h_align=ColAlign::End>
                        <ButtonGroup>
                            {move || {
                                let page_options = page_options.get();
                                page_options
                                    .options
                                    .into_iter()
                                    .map(|page_number| {
                                        view! {
                                            <Button
                                                variant=ButtonVariant::Filled
                                                color=ButtonColor::Secondary
                                                disabled=page_number.is_none()
                                                // TODO: Use signal::derive instead?
                                                //active=MaybeSignal::from(
                                                //    page_number == Some(page_options.for_current_page),
                                                //)
                                                on_press=move |_| {
                                                    if let Some(number) = page_number {
                                                        set_current_page.run(number)
                                                    }
                                                }
                                            >
                                                {match page_number {
                                                    Some(page_number) => Cow::Owned(format!("{}", page_number.0)),
                                                    None => Cow::Borrowed("\u{2026}"),
                                                }}
                                            </Button>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </ButtonGroup>
                    </Col>
                </Row>
            </Grid>
        </Show>
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ItemsPerPageEntry {
    items_per_page: ItemsPerPage,
    all: bool,
}

impl ItemsPerPageEntry {
    pub fn some(items_per_page: ItemsPerPage) -> Self {
        Self {
            items_per_page,
            all: false,
        }
    }

    pub fn all(items_per_page: ItemsPerPage) -> Self {
        Self {
            items_per_page,
            all: true,
        }
    }
}

impl Display for ItemsPerPageEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.all {
            true => f.write_fmt(format_args!("All ({})", self.items_per_page.0)),
            false => f.write_fmt(format_args!("{}", self.items_per_page.0)),
        }
    }
}

#[derive(Debug, Clone)]
struct PageOptions {
    /// Options for pages to which navigation can occur.
    ///
    /// Occurrences of `None` describe "gaps" in the available numbers to go to. These can either
    /// be ignored or rendered using a special non-interactable symbol.
    ///
    /// Example:
    /// ```ignore
    /// [
    ///     Some(1),
    ///     Some(2),
    ///     None,
    ///     Some(11),
    ///     Some(12),
    ///     Some(13), // Current page!
    ///     Some(14),
    ///     Some(15),
    ///     None,
    ///     Some(41),
    ///     Some(42),
    /// ]
    /// ```
    options: Vec<Option<PageNr>>,
}

impl PageOptions {
    fn new(page_count: u64, current_page: u64) -> Self {
        let mut options: Vec<Option<PageNr>>;
        // Just return all available pages if there are not too many of them.
        if page_count <= 10 {
            options = Vec::with_capacity(page_count as usize);
            for i in 1..=page_count {
                options.push(Some(PageNr(i)));
            }
        }
        // Single ... at the right. Start of page spectrum.
        else if current_page <= 5 {
            options = vec![
                Some(PageNr(1)),
                Some(PageNr(2)),
                Some(PageNr(3)),
                Some(PageNr(4)),
                Some(PageNr(5)),
                Some(PageNr(6)),
                Some(PageNr(7)),
                None,
                Some(PageNr(page_count - 1)),
                Some(PageNr(page_count)),
            ];
        }
        // With ... at the left and right. In the middle of the available pages.
        else if current_page > 5 && current_page < page_count - 4 {
            options = vec![
                Some(PageNr(1)),
                Some(PageNr(2)),
                None,
                Some(PageNr(current_page - 2)),
                Some(PageNr(current_page - 1)),
                Some(PageNr(current_page)),
                Some(PageNr(current_page + 1)),
                Some(PageNr(current_page + 2)),
                None,
                Some(PageNr(page_count - 1)),
                Some(PageNr(page_count)),
            ];
        }
        // Single ... at the left. End of page spectrum.
        else if current_page >= page_count - 4 {
            options = vec![
                Some(PageNr(1)),
                Some(PageNr(2)),
                None,
                Some(PageNr(page_count - 6)),
                Some(PageNr(page_count - 5)),
                Some(PageNr(page_count - 4)),
                Some(PageNr(page_count - 3)),
                Some(PageNr(page_count - 2)),
                Some(PageNr(page_count - 1)),
                Some(PageNr(page_count)),
            ];
        } else {
            panic!("Unreachable!");
        }
        Self { options }
    }
}
