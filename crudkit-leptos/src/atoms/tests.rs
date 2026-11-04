//! Tests of the markup atoms render: their elements, default classes, and data attributes.

use crate::config::{CrudInstanceConfig, FieldRenderer, FieldRendererRegistry};
use crate::hooks::form::FieldTarget;
use crate::hooks::tabs::TabLocation;
use crate::instance::{provide_crud_instance, provide_view_context};
use crate::prelude::*;
use crate::test_support::items::{
    CreateItemField, FakeServer, ReadItemField, config, instance_input,
};
use crate::test_support::memory::{MemoryPaging, MemorySource};
use crate::test_support::with_crud_manager;
use assertr::prelude::*;
use leptonic::atoms::tabs::TabList;
use leptos::context::Provider;
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
fn table_headers_sort_only_by_fields_the_data_source_orders_by() {
    // The data source in memory orders by name only.
    let source = MemorySource::with_items(0, MemoryPaging::Offset);
    let column = |field: ReadItemField, name: &'static str| {
        Header::showing(
            field,
            HeaderOptions {
                display_name: name.into(),
                ordering_allowed: true,
                ..HeaderOptions::default()
            },
        )
    };
    let config = CrudInstanceConfig {
        list_columns: vec![
            column(ReadItemField::Id, "ID"),
            column(ReadItemField::Name, "Name"),
        ],
        ..source.config()
    };
    let html = render_in_instance(config, || {
        view! {
            <CrudList>
                <CrudTable>
                    <CrudTableHeader>
                        <CrudTableHeaderRow>
                            <CrudTableColumnHeaders />
                        </CrudTableHeaderRow>
                    </CrudTableHeader>
                </CrudTable>
            </CrudList>
        }
    });
    let headers = elements_with_class(&html, "crudkit-TableColumnHeader");
    assert_that!(headers.len()).is_equal_to(2);
    assert_that!(headers[0]).does_not_contain("data-allows-sorting");
    assert_that!(headers[1]).contains("data-allows-sorting");
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

#[test]
fn form_errors_are_empty_alerts_until_a_save_fails() {
    let html = render_in_create_form(items_config(), || {
        view! { <CrudFormError classes="app-error" /> }
    });
    let errors = elements_with_class(&html, "crudkit-FormError");
    assert_that!(errors.len()).is_equal_to(1);
    assert_that!(errors[0]).starts_with("div");
    assert_that!(errors[0]).contains("role=\"alert\"");
    assert_that!(errors[0]).contains("app-error");
    assert_that!(errors[0]).does_not_contain("data-kind");
}

/// Renders a `CrudFormError` in an edit form whose last save failed with `error`, if any, and
/// whose fields have `conflicts`.
fn render_failed_save(error: Option<RequestError>, conflicts: Vec<CrudFieldConflict>) -> String {
    render_in_instance(items_config(), move || {
        let form = CrudFormHandle {
            kind: CrudEntityViewKind::Update,
            status: Signal::stored(CrudEntityLoadStatus::Ready),
            is_dirty: Signal::stored(true),
            has_errors: Signal::stored(false),
            is_saving: Signal::stored(false),
            can_save: Signal::stored(true),
            save: None,
            violations: Signal::stored(None),
            follow_up: CrudSaveFollowUp::Stay,
            draft: Signal::stored(None),
            save_error: Signal::stored(error),
            problems: Signal::stored(Vec::new()),
            conflicts: Signal::stored(conflicts),
            reload: Some(Callback::new(|()| {})),
            field_targets: Signal::stored(std::collections::HashMap::new()),
            error_summaries: StoredValue::new(0),
            field_type: std::any::TypeId::of::<DynUpdateField>(),
            submits: false,
        };
        view! {
            <Provider value=Some(form)>
                <CrudFormError />
            </Provider>
        }
    })
}

#[test]
fn form_errors_offer_the_current_state_of_an_outdated_entity() {
    let html = render_failed_save(
        Some(RequestError::Outdated("changed".to_owned())),
        Vec::new(),
    );

    let errors = elements_with_class(&html, "crudkit-FormError");
    assert_that!(errors.len()).is_equal_to(1);
    assert_that!(errors[0]).contains("data-kind=\"outdated\"");
    assert_that!(html.as_str()).contains(CrudUiTexts::default().outdated.as_ref());
    let reload = elements_with_class(&html, "crudkit-ReloadButton");
    assert_that!(reload.len()).is_equal_to(1);
    assert_that!(reload[0]).contains("leptonic-Button");
}

/// The tabs of the surrounding form's `CrudFormTabs` in the tests.
fn general_and_more() -> Vec<CrudTab> {
    vec![
        CrudTab {
            id: TabId::from("general"),
            label: "General".to_owned(),
        },
        CrudTab {
            id: TabId::from("more"),
            label: "More".to_owned(),
        },
    ]
}

/// Makes the last save of the surrounding create form fail with problems of the fields `name`
/// and `active`, and renders `name` in the tab `more`, as a field in that tab's panel would.
#[component]
fn FailedSave() -> impl IntoView {
    let form = expect_context::<super::form::FormContext<DynCreateField>>();
    form.state.register_field(
        "name".to_owned(),
        FieldTarget {
            dom_id: leptonic::Id::from("name-input"),
            label: Some("Name".to_owned()),
            tabs: vec![TabLocation {
                instance: use_crud_instance(),
                group: general_and_more().into_iter().map(|tab| tab.id).collect(),
                tab: TabId::from("more"),
            }],
        },
    );
    form.state
        .record_save_error(Some(RequestError::InvalidFields {
            message: "The item is invalid.".to_owned(),
            problems: vec![
                CrudFieldProblem::new("name", "Another item has this name."),
                CrudFieldProblem::new("active", "Must be active."),
            ],
        }));
}

#[test]
fn error_summaries_list_every_problem_and_mark_the_tabs_of_their_fields() {
    let html = render_in_create_form(items_config(), move || {
        view! {
            <FailedSave />
            // A switch shows no error text of its own.
            <CrudField field=CreateItemField::Active>
                <CrudSwitch />
            </CrudField>
            <CrudFormTabs tabs=general_and_more()>
                <TabList>
                    <CrudFormTab id=TabId::from("general") />
                    <CrudFormTab id=TabId::from("more") />
                </TabList>
                <CrudFormTabPanel id=TabId::from("general") should_force_mount=true>
                    "Nothing to fix"
                </CrudFormTabPanel>
                <CrudFormTabPanel id=TabId::from("more") should_force_mount=true>
                    "The name"
                </CrudFormTabPanel>
            </CrudFormTabs>
            <CrudFormError />
        }
    });

    assert_that!(elements_with_class(&html, "crudkit-FormError")[0])
        .contains("data-kind=\"invalid\"");
    assert_that!(html.as_str()).contains("The item is invalid.");
    // Every problem links to its field, by the field's label or else its name.
    let links = html
        .split("<a ")
        .skip(1)
        .map(|link| link.split("</a>").next().unwrap_or_default())
        .collect::<Vec<_>>();
    assert_that!(links.len()).is_equal_to(2);
    assert_that!(links[0]).is_equal_to("href=\"#name-input\">Name: Another item has this name.");
    assert_that!(links[1]).starts_with("href=\"#crudkit-field-");
    assert_that!(links[1]).ends_with(">active: Must be active.");
    let tabs = elements_with_class(&html, "crudkit-FormTab");
    assert_that!(tabs.len()).is_equal_to(2);
    assert_that!(tabs[0]).does_not_contain("data-invalid");
    assert_that!(tabs[1]).contains("data-invalid");
}

/// The tabs that the field `name` of the surrounding create form is rendered in, as text.
#[component]
fn TabsOfName() -> impl IntoView {
    let form = expect_context::<super::form::FormContext<DynCreateField>>();
    form.state
        .field_targets()
        .get_untracked()
        .get("name")
        .map(|target| {
            target
                .tabs
                .iter()
                .map(|location| location.tab.to_string())
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default()
}

#[test]
fn fields_in_tab_panels_know_their_tabs() {
    let html = render_in_create_form(items_config(), move || {
        view! {
            <CrudFormTabs tabs=general_and_more()>
                <CrudFormTabPanel id=TabId::from("more") should_force_mount=true>
                    <CrudField field=CreateItemField::Name>
                        <span class="app-tabs">
                            <TabsOfName />
                        </span>
                    </CrudField>
                </CrudFormTabPanel>
            </CrudFormTabs>
        }
    });

    assert_that!(html.as_str()).contains("<span class=\"app-tabs\">more</span>");
}

#[test]
fn error_summaries_explain_conflicts_found_by_a_reload() {
    let html = render_failed_save(
        None,
        vec![CrudFieldConflict {
            field: "tags".to_owned(),
            pointer: "/tags/team".to_owned(),
            theirs: Some(serde_json::json!("blue")),
        }],
    );

    let texts = CrudUiTexts::default();
    assert_that!(elements_with_class(&html, "crudkit-FormError")[0])
        .contains("data-kind=\"conflict\"");
    assert_that!(html.as_str()).contains(texts.conflicts.as_ref());
    assert_that!(html.as_str()).contains(format!(
        "<li>tags/team: {}</li>",
        (texts.changed_meanwhile)("blue").replace('"', "&quot;")
    ));
}

#[test]
fn form_errors_explain_other_failures() {
    let html = render_failed_save(
        Some(RequestError::Forbidden("not yours".to_owned())),
        Vec::new(),
    );

    assert_that!(elements_with_class(&html, "crudkit-FormError")[0])
        .contains("data-kind=\"failed\"");
    assert_that!(html.as_str()).contains("Permission denied: not yours");
    assert_that!(elements_with_class(&html, "crudkit-ReloadButton")).is_empty();
}

#[test]
#[should_panic(
    expected = "`CrudReloadButton` must be rendered inside an edit form or details, not in a create form"
)]
fn reload_buttons_need_an_entity_to_reload() {
    render_in_create_form(items_config(), || view! { <CrudReloadButton /> });
}
