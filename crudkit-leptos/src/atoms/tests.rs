//! Tests of the markup atoms render: their elements, default classes, and data attributes.

use crate::config::{CrudInstanceConfig, FieldRenderer, FieldRendererRegistry};
use crate::instance::{provide_crud_instance, provide_view_context};
use crate::prelude::*;
use crate::test_support::items::{
    CreateItemField, FakeServer, ReadItemField, config, instance_input,
};
use crate::test_support::with_crud_manager;
use assertr::prelude::*;
use leptonic::atoms::prelude::TabList;
use leptos::prelude::*;

/// Renders `view` to HTML inside a mounted `items` instance configured by `config`.
fn render_in_instance<V: IntoView + 'static>(
    config: CrudInstanceConfig,
    view: impl FnOnce() -> V,
) -> String {
    let mut html = String::new();
    with_crud_manager(|| {
        let ctx = provide_crud_instance(instance_input("items", config));
        provide_view_context(&ctx, ctx.navigation);
        html = view().into_view().to_html();
    });
    html
}

/// Renders `view` inside a create form of an `items` instance configured by `config`.
fn render_in_create_form<V: IntoView + 'static>(
    config: CrudInstanceConfig,
    view: impl Fn() -> V + Send + Sync + 'static,
) -> String {
    render_in_instance(config, move || {
        view! { <CrudCreateForm>{view()}</CrudCreateForm> }
    })
}

/// Returns the opening tags of the elements in `html` carrying `class`.
fn elements_with_class<'a>(html: &'a str, class: &str) -> Vec<&'a str> {
    html.split('<')
        .filter(|tag| {
            tag.split('>').next().is_some_and(|tag| {
                tag.split("class=\"")
                    .nth(1)
                    .and_then(|classes| classes.split('"').next())
                    .is_some_and(|classes| classes.split(' ').any(|name| name == class))
            })
        })
        .collect()
}

fn items_config() -> CrudInstanceConfig {
    config(&FakeServer::with_items(0))
}

#[test]
fn values_render_one_span_with_their_kind() {
    let html = render_in_instance(items_config(), || {
        view! { <CrudValue value=Signal::stored(Value::I32(1981)) classes="app-value" /> }
    });
    let spans = elements_with_class(&html, "crudkit-Value");
    assert_that!(spans.len()).is_equal_to(1);
    assert_that!(spans[0]).starts_with("span");
    assert_that!(spans[0]).contains("app-value");
    assert_that!(spans[0]).contains("data-kind=\"number\"");
    assert_that!(spans[0]).does_not_contain("data-empty");
    assert_that!(html.as_str()).contains("1981");
}

#[test]
fn absent_values_are_marked_empty_without_a_kind() {
    let html = render_in_instance(items_config(), || {
        view! { <CrudValue value=Signal::stored(Value::Null) /> }
    });
    let spans = elements_with_class(&html, "crudkit-Value");
    assert_that!(spans.len()).is_equal_to(1);
    assert_that!(spans[0]).contains("data-empty");
    assert_that!(spans[0]).does_not_contain("data-kind");
}

#[test]
fn buttons_keep_leptonics_class_and_add_their_own_and_the_callers() {
    let html = render_in_instance(items_config(), || {
        view! { <CrudCreateButton classes="app-button" /> }
    });
    let buttons = elements_with_class(&html, "crudkit-CreateButton");
    assert_that!(buttons.len()).is_equal_to(1);
    assert_that!(buttons[0]).starts_with("button");
    assert_that!(buttons[0]).contains("leptonic-Button");
    assert_that!(buttons[0]).contains("app-button");
    assert_that!(buttons[0]).does_not_contain("data-disabled");
}

#[test]
fn create_forms_render_one_form_element() {
    let html = render_in_create_form(items_config(), || ());
    let forms = elements_with_class(&html, "crudkit-CreateForm");
    assert_that!(forms.len()).is_equal_to(1);
    assert_that!(forms[0]).starts_with("form");
    assert_that!(forms[0]).does_not_contain("data-dirty");
    assert_that!(forms[0]).does_not_contain("data-saving");
}

#[test]
fn field_controls_default_to_a_labelled_text_field() {
    let html = render_in_create_form(items_config(), || {
        view! {
            <CrudField field=CreateItemField::Name options=FieldOptions {
                label: Some(Label::new("Name")),
                ..FieldOptions::default()
            }>
                <CrudFieldControl />
            </CrudField>
        }
    });
    assert_that!(elements_with_class(&html, "crudkit-TextField").len()).is_equal_to(1);
    let labels = elements_with_class(&html, "crudkit-FieldLabel");
    assert_that!(labels.len()).is_equal_to(1);
    assert_that!(labels[0]).starts_with("label");
    assert_that!(html.as_str()).contains(">Name<");
}

#[test]
fn registered_renderers_can_wrap_the_default_field_control() {
    let mut config = items_config();
    config.create_field_renderer = FieldRendererRegistry::default().register(
        CreateItemField::Name,
        FieldRenderer::new(|_| view! { <div class="app-wrapper"><CrudFieldControl /></div> }),
    );
    let html = render_in_create_form(config, || {
        view! {
            <CrudField field=CreateItemField::Name>
                <CrudFieldControl />
            </CrudField>
        }
    });
    assert_that!(elements_with_class(&html, "app-wrapper").len()).is_equal_to(1);
    assert_that!(elements_with_class(&html, "crudkit-TextField").len()).is_equal_to(1);
}

#[test]
fn display_fields_are_groups_named_by_their_label() {
    let html = render_in_create_form(items_config(), || {
        view! {
            <CrudField
                field=CreateItemField::Name
                mode=FieldMode::Display
                options=FieldOptions {
                    label: Some(Label::new("Name")),
                    ..FieldOptions::default()
                }
            >
                <CrudFieldControl />
            </CrudField>
        }
    });
    let groups = elements_with_class(&html, "crudkit-DisplayField");
    assert_that!(groups.len()).is_equal_to(1);
    assert_that!(groups[0]).contains("role=\"group\"");
    let labels = elements_with_class(&html, "crudkit-FieldLabel");
    assert_that!(labels.len()).is_equal_to(1);
    assert_that!(labels[0]).starts_with("span");
    let label_id = labels[0]
        .split("id=\"")
        .nth(1)
        .and_then(|id| id.split('"').next())
        .expect("the label has an id");
    assert_that!(groups[0]).contains(format!("aria-labelledby=\"{label_id}\"").as_str());
    assert_that!(elements_with_class(&html, "crudkit-FieldValue").len()).is_equal_to(1);
}

#[test]
fn list_status_reports_loading_as_a_status() {
    let html = render_in_instance(items_config(), || {
        view! {
            <CrudList>
                <CrudListStatus />
            </CrudList>
        }
    });
    let statuses = elements_with_class(&html, "crudkit-ListStatus");
    assert_that!(statuses.len()).is_equal_to(1);
    assert_that!(statuses[0]).contains("role=\"status\"");
    assert_that!(statuses[0]).contains("data-status=\"loading\"");
}

#[test]
fn selection_counts_are_empty_live_regions_without_a_selection() {
    let html = render_in_instance(items_config(), || {
        view! {
            <CrudList>
                <CrudSelectionCount />
            </CrudList>
        }
    });
    let counts = elements_with_class(&html, "crudkit-SelectionCount");
    assert_that!(counts.len()).is_equal_to(1);
    assert_that!(counts[0]).contains("role=\"status\"");
    assert_that!(counts[0]).contains("data-empty");
    assert_that!(html.as_str()).contains("0 ausgewählt");
}

#[test]
#[should_panic(
    expected = "`CrudDeleteButton` must be rendered inside a table row or an entity form"
)]
fn atoms_outside_of_their_container_panic() {
    render_in_instance(items_config(), || view! { <CrudDeleteButton /> });
}

#[test]
fn form_tabs_key_their_tabs_and_panels_by_tab_id() {
    let tabs = vec![
        CrudTab {
            id: TabId::from("contact"),
            label: "Contact".to_owned(),
        },
        CrudTab {
            id: TabId::from("history"),
            label: "History".to_owned(),
        },
    ];
    let html = render_in_create_form(items_config(), move || {
        view! {
            <CrudFormTabs tabs=tabs.clone()>
                <TabList>
                    <CrudFormTab id=TabId::from("contact") />
                    <CrudFormTab id=TabId::from("history") />
                </TabList>
                <CrudFormTabPanel id=TabId::from("contact")>"Contact details"</CrudFormTabPanel>
                <CrudFormTabPanel id=TabId::from("history")>"Past clubs"</CrudFormTabPanel>
            </CrudFormTabs>
        }
    });
    assert_that!(elements_with_class(&html, "crudkit-FormTabs").len()).is_equal_to(1);
    assert_that!(elements_with_class(&html, "crudkit-FormTab").len()).is_equal_to(2);
    assert_that!(html.as_str()).contains(">Contact<");
    // The first tab is selected, so only its panel is shown.
    assert_that!(html.as_str()).contains("Contact details");
    assert_that!(html.as_str()).does_not_contain("Past clubs");
}

#[test]
fn save_buttons_without_a_follow_up_submit_their_form() {
    let html = render_in_create_form(items_config(), || {
        view! {
            <CrudSaveButton />
            <CrudSaveButton follow_up=CrudSaveFollowUp::Return classes="app-return" />
        }
    });
    let buttons = elements_with_class(&html, "crudkit-SaveButton");
    assert_that!(buttons.len()).is_equal_to(2);
    assert_that!(buttons[0]).contains("type=\"submit\"");
    assert_that!(buttons[1]).contains("type=\"button\"");
}

#[test]
fn forms_inside_forms_render_no_nested_form_element() {
    let html = render_in_create_form(items_config(), || {
        view! {
            <CrudCreateForm classes="app-nested">
                <CrudSaveButton classes="app-nested-save" />
            </CrudCreateForm>
        }
    });
    assert_that!(html.matches("<form").count()).is_equal_to(1);
    let nested = elements_with_class(&html, "app-nested");
    assert_that!(nested.len()).is_equal_to(1);
    assert_that!(nested[0]).starts_with("div");
    assert_that!(nested[0]).contains("role=\"group\"");
    // A submit button would submit the outer form.
    let save = elements_with_class(&html, "app-nested-save");
    assert_that!(save[0]).contains("type=\"button\"");
}

/// The `items` configuration listing the items' names in a sortable column.
fn listed_items_config() -> CrudInstanceConfig {
    CrudInstanceConfig {
        list_columns: vec![Header::showing(
            ReadItemField::Name,
            HeaderOptions {
                display_name: "Name".into(),
                ordering_allowed: true,
                ..HeaderOptions::default()
            },
        )],
        ..items_config()
    }
}

#[test]
fn table_headers_expose_their_column() {
    let html = render_in_instance(listed_items_config(), || {
        view! {
            <CrudList>
                <CrudTable selectable=true with_actions=true>
                    <CrudTableHeader>
                        <CrudTableHeaderRow>
                            <CrudTableSelectAllHeader>
                                <CrudSelectAllCheckbox />
                            </CrudTableSelectAllHeader>
                            <CrudTableColumnHeaders />
                            <CrudTableActionsHeader />
                        </CrudTableHeaderRow>
                    </CrudTableHeader>
                </CrudTable>
            </CrudList>
        }
    });
    let tables = elements_with_class(&html, "crudkit-Table");
    assert_that!(tables.len()).is_equal_to(1);
    assert_that!(tables[0]).contains("role=\"grid\"");
    assert_that!(tables[0]).contains("aria-label=\"items\"");
    let headers = elements_with_class(&html, "crudkit-TableColumnHeader");
    assert_that!(headers.len()).is_equal_to(1);
    assert_that!(headers[0]).starts_with("th");
    assert_that!(headers[0]).contains("data-kind=\"text\"");
    assert_that!(headers[0]).contains("data-allows-sorting");
    assert_that!(headers[0]).does_not_contain("data-sort-direction");
    assert_that!(html.as_str()).contains(">Name<");
    assert_that!(elements_with_class(&html, "crudkit-TableSelectAllHeader").len()).is_equal_to(1);
    let checkboxes = elements_with_class(&html, "crudkit-SelectAllCheckbox");
    assert_that!(checkboxes.len()).is_equal_to(1);
    assert_that!(checkboxes[0]).contains("aria-label=\"Alle auswählen\"");
    assert_that!(elements_with_class(&html, "crudkit-TableActionsHeader").len()).is_equal_to(1);
}

#[test]
#[should_panic(
    expected = "`CrudTableSelectAllHeader` must be rendered inside a `CrudTable` with `selectable`"
)]
fn selection_atoms_need_a_selectable_table() {
    render_in_instance(listed_items_config(), || {
        view! {
            <CrudList>
                <CrudTable>
                    <CrudTableHeader>
                        <CrudTableHeaderRow>
                            <CrudTableSelectAllHeader>"All"</CrudTableSelectAllHeader>
                        </CrudTableHeaderRow>
                    </CrudTableHeader>
                </CrudTable>
            </CrudList>
        }
    });
}

#[test]
fn pagination_is_a_labelled_navigation() {
    let html = render_in_instance(items_config(), || {
        view! {
            <CrudList>
                <CrudPagination>
                    <CrudPreviousPageButton />
                    <CrudPageButtons />
                    <CrudNextPageButton />
                </CrudPagination>
            </CrudList>
        }
    });
    let navs = elements_with_class(&html, "crudkit-Pagination");
    assert_that!(navs.len()).is_equal_to(1);
    assert_that!(navs[0]).starts_with("nav");
    assert_that!(navs[0]).contains("aria-label=");
    // Without a known count, there are no pages to go to.
    let previous = elements_with_class(&html, "crudkit-PreviousPageButton");
    assert_that!(previous.len()).is_equal_to(1);
    assert_that!(previous[0]).contains("disabled");
}

#[test]
#[should_panic(expected = "`CrudConfirmButton` must be rendered inside a confirmation dialog")]
fn confirmation_atoms_need_a_dialog() {
    render_in_instance(items_config(), || view! { <CrudConfirmButton /> });
}

#[test]
fn closed_confirmation_dialogs_render_nothing() {
    let html = render_in_instance(items_config(), || {
        view! {
            <CrudDeleteDialog classes="app-dialog">
                <CrudConfirmationTitle />
            </CrudDeleteDialog>
        }
    });
    assert_that!(elements_with_class(&html, "app-dialog").len()).is_equal_to(0);
}
