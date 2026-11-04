//! Tests of the state hooks against an in-memory CrudKit server.

use crate::config::CrudInstanceConfig;
use crate::hooks::form::{
    CrudSaveFollowUp, UseCrudCreateFormInput, UseCrudEditFormInput, use_crud_create_form,
    use_crud_edit_form,
};
use crate::hooks::instance::use_crud_navigation;
use crate::hooks::list::CrudListState;
use crate::hooks::list::use_crud_list_state;
use crate::hooks::notify::{
    CrudNotification, CrudNotificationKind, CrudNotifier, provide_crud_notifier,
};
use crate::instance::{provide_crud_instance, provide_view_context};
use crate::prelude::*;
use crate::test_support::items::*;
use crate::test_support::memory::{MemoryOperation, MemoryPaging, MemorySource, TAKEN_NAME};
use crate::test_support::{CrudTestResponse, CrudTestServer, settle, with_crud_manager};
use assertr::prelude::*;
use indexmap::IndexMap;
use leptos::prelude::*;
use leptos::reactive::owner::Owner;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[test]
fn list_loads_pages_and_clamps_the_page_when_the_page_size_grows() {
    let server = FakeServer::with_items(5);
    with_instance(&server, |_ctx| {
        let list = use_crud_list_state();
        settle();
        assert_that!(list.rows.get_untracked().loaded().map(|rows| rows.len()))
            .is_equal_to(Some(2));
        assert_that!(list.item_count.get_untracked()).is_equal_to(Some(5));
        assert_that!(list.pagination.page_count.get_untracked()).is_equal_to(3);

        list.pagination.set_page(PageNr(3));
        settle();
        assert_that!(list.rows.get_untracked().loaded().map(|rows| rows.len()))
            .is_equal_to(Some(1));

        list.pagination.set_items_per_page(ItemsPerPage(10));
        settle();
        assert_that!(list.pagination.page.get_untracked()).is_equal_to(PageNr(1));
        assert_that!(list.rows.get_untracked().loaded().map(|rows| rows.len()))
            .is_equal_to(Some(5));
    });
}

#[test]
fn pages_beyond_the_last_one_move_to_the_last_page() {
    let server = FakeServer::with_items(5);
    with_instance(&server, |_ctx| {
        let list = use_crud_list_state();
        settle();

        // E.g. after deletions elsewhere, or a page restored from an earlier visit.
        list.pagination.set_page(PageNr(9));
        settle();
        assert_that!(list.pagination.page.get_untracked()).is_equal_to(PageNr(3));
        assert_that!(list.rows.get_untracked().loaded().map(|rows| rows.len()))
            .is_equal_to(Some(1));
    });
}

#[test]
fn selection_drops_entities_that_are_no_longer_displayed() {
    let server = FakeServer::with_items(5);
    with_instance(&server, |_ctx| {
        let list = use_crud_list_state();
        settle();
        list.selection.toggle_all();
        assert_that!(list.selection.all_selected.get_untracked()).is_true();

        list.pagination.next();
        settle();
        assert_that!(list.selection.count.get_untracked()).is_equal_to(0);
        assert_that!(list.selection.all_selected.get_untracked()).is_false();

        list.selection.toggle(item(3));
        assert_that!(list.selection.is_selected(&item(3))).is_true();
    });
}

#[test]
fn ordering_toggles_are_sent_with_the_next_request() {
    let server = FakeServer::with_items(3);
    with_instance(&server, |_ctx| {
        let list = use_crud_list_state();
        settle();
        list.ordering
            .toggle(DynReadField::from(ReadItemField::Name), false);
        list.ordering
            .toggle(DynReadField::from(ReadItemField::Name), false);
        settle();
        let last_request = server
            .requests_to("read-many")
            .pop()
            .expect("a read-many request");
        // Ordering keys use the same field names as conditions.
        assert_that!(last_request["order_by"].clone())
            .is_equal_to(serde_json::json!({ "name": "desc" }));
        assert_that!(
            list.ordering
                .order_of(&DynReadField::from(ReadItemField::Name))
        )
        .is_equal_to(Some(Order::Desc));
    });
}

#[test]
fn create_form_tracks_changes_and_opens_the_created_entity() {
    let server = FakeServer::with_items(0);
    with_instance(&server, |ctx| {
        let create = use_crud_create_form(UseCrudCreateFormInput::default());
        assert_that!(create.form.is_dirty.get_untracked()).is_false();

        create.form.set_field(
            DynCreateField::from(CreateItemField::Name),
            Ok(Value::String("new".to_owned())),
        );
        assert_that!(create.form.is_dirty.get_untracked()).is_true();
        assert_that!(
            create
                .form
                .field(&DynCreateField::from(CreateItemField::Name))
                .and_then(|field| field.get_untracked().as_string().cloned())
        )
        .is_equal_to(Some("new".to_owned()));

        create.form.set_field(
            DynCreateField::from(CreateItemField::Name),
            Err("invalid".to_owned()),
        );
        assert_that!(
            create
                .form
                .field_error(DynCreateField::from(CreateItemField::Name))
                .get_untracked()
        )
        .is_equal_to(Some("invalid".to_owned()));

        // A draft with rejected input is not saved, because it does not reflect what the user sees.
        assert_that!(create.can_save.get_untracked()).is_false();
        create.save.run(CrudSaveFollowUp::Edit);
        settle();
        assert_that!(server.requests_to("create-one")).is_empty();

        create.form.set_field(
            DynCreateField::from(CreateItemField::Name),
            Ok(Value::String("new".to_owned())),
        );
        assert_that!(create.can_save.get_untracked()).is_true();
        create.save.run(CrudSaveFollowUp::Edit);
        settle();
        let request = server
            .requests_to("create-one")
            .pop()
            .expect("a create-one request");
        assert_that!(request["entity"]["name"].as_str()).is_equal_to(Some("new"));
        let current = ctx.navigation.current().get_untracked();
        assert_that!(current.name.as_str()).is_equal_to(crudkit_web::view::EDIT_VIEW);
        assert_that!(current.subject.is_some()).is_true();
    });
}

#[test]
fn edit_form_loads_saves_and_resets_its_dirty_state() {
    let server = FakeServer::with_items(1);
    with_instance(&server, |_ctx| {
        let id = item(1).id();
        let edit = use_crud_edit_form(edit_input(id));
        settle();
        assert_that!(edit.entity.get_untracked().loaded().is_some()).is_true();
        assert_that!(edit.form.is_dirty.get_untracked()).is_false();
        assert_that!(edit.can_save.get_untracked()).is_false();

        edit.form.set_field(
            DynUpdateField::from(ItemField::Name),
            Ok(Value::String("renamed".to_owned())),
        );
        assert_that!(edit.can_save.get_untracked()).is_true();

        edit.save.run(CrudSaveFollowUp::Stay);
        settle();
        let request = server
            .requests_to("update-one")
            .pop()
            .expect("an update-one request");
        assert_that!(request["entity"]["name"].as_str()).is_equal_to(Some("renamed"));
        assert_that!(edit.form.is_dirty.get_untracked()).is_false();
    });
}

#[test]
fn confirmed_deletions_are_sent_and_reported() {
    let server = FakeServer::with_items(3);
    let notifications = Arc::new(Mutex::new(Vec::<CrudNotification>::new()));
    with_crud_manager(|| {
        let recorded = notifications.clone();
        provide_crud_notifier(CrudNotifier::new(move |notification| {
            recorded.lock().expect("lock").push(notification);
        }));
        provide_crud_instance(instance_input("items", config(&server)));

        let deletion = crate::hooks::delete::use_crud_delete();
        deletion.request(item(2));
        assert_that!(deletion.pending.get_untracked().is_some()).is_true();
        deletion.confirm();
        settle();
        assert_that!(server.requests_to("delete-by-id").len()).is_equal_to(1);
        assert_that!(deletion.pending.get_untracked().is_none()).is_true();

        deletion.request_many(Arc::new(Vec::new()));
        assert_that!(deletion.pending_many.get_untracked().is_none()).is_true();

        deletion.request_many(Arc::new(vec![item(1), item(3)]));
        deletion.confirm_many();
        settle();
        let request = server
            .requests_to("delete-many")
            .pop()
            .expect("a delete-many request");
        assert_that!(request["condition"].to_string()).contains("Any");
        assert_that!(deletion.pending_many.get_untracked().is_none()).is_true();
    });

    let kinds = notifications
        .lock()
        .expect("lock")
        .iter()
        .map(|notification| notification.kind)
        .collect::<Vec<_>>();
    assert_that!(kinds).is_equal_to(vec![
        CrudNotificationKind::Success,
        CrudNotificationKind::Success,
    ]);
    // The instance names itself as the origin of what it reports.
    let origins = notifications
        .lock()
        .expect("lock")
        .iter()
        .map(|notification| {
            notification
                .origin
                .as_ref()
                .map(|origin| (origin.instance_name, origin.resource_name.clone()))
        })
        .collect::<Vec<_>>();
    assert_that!(origins).is_equal_to(vec![Some(("items", "items".to_owned())); 2]);
}

#[test]
fn editing_a_loaded_form_does_not_renotify_readiness() {
    let server = FakeServer::with_items(1);
    with_instance(&server, |_ctx| {
        let edit = use_crud_edit_form(edit_input(item(1).id()));
        settle();
        let notifications = Arc::new(Mutex::new(0_usize));
        let counter = notifications.clone();
        let is_ready = edit.form.is_ready;
        Effect::new(move |_| {
            is_ready.track();
            *counter.lock().expect("lock") += 1;
        });
        settle();
        for name in ["a", "ab", "abc"] {
            edit.form.set_field(
                DynUpdateField::from(ItemField::Name),
                Ok(Value::String(name.to_owned())),
            );
            settle();
        }
        // Views keyed on readiness re-render only when an entity (re)loads, not on every keystroke.
        assert_that!(*notifications.lock().expect("lock")).is_equal_to(1);
    });
}

#[test]
fn saving_continues_from_the_servers_version_of_the_entity() {
    let server = FakeServer::with_items(1);
    with_instance(&server, |_ctx| {
        let edit = use_crud_edit_form(edit_input(item(1).id()));
        settle();
        let name = DynUpdateField::from(ItemField::Name);
        edit.form
            .set_field(name.clone(), Ok(Value::String("  padded  ".to_owned())));
        edit.save.run(CrudSaveFollowUp::Stay);
        settle();

        assert_that!(edit.form.is_dirty.get_untracked()).is_false();
        assert_that!(
            edit.form
                .field(&name)
                .and_then(|field| field.get_untracked().as_string().cloned())
        )
        .is_equal_to(Some("padded".to_owned()));
    });
}

#[test]
fn staying_after_creating_starts_a_fresh_draft() {
    let server = FakeServer::with_items(0);
    with_instance(&server, |_ctx| {
        let create = use_crud_create_form(UseCrudCreateFormInput::default());
        let name = DynCreateField::from(CreateItemField::Name);
        create
            .form
            .set_field(name.clone(), Ok(Value::String("first".to_owned())));
        create.save.run(CrudSaveFollowUp::Stay);
        settle();

        assert_that!(server.requests_to("create-one").len()).is_equal_to(1);
        assert_that!(create.form.is_dirty.get_untracked()).is_false();
        assert_that!(
            create
                .form
                .field(&name)
                .and_then(|field| field.get_untracked().as_string().cloned())
        )
        .is_equal_to(Some(String::new()));
    });
}

#[test]
fn tab_groups_remember_their_selection_independently() {
    let server = FakeServer::with_items(0);
    with_instance(&server, |ctx| {
        let details = [TabId::from("general"), TabId::from("contact")];
        let extras = [TabId::from("notes"), TabId::from("history")];
        let tabs = crate::hooks::tabs::use_crud_tab_selection(details.to_vec());

        // Without a selection, the first tab is selected.
        assert_that!(ctx.selected_tab(&details)).is_none();
        assert_that!(tabs.selected_key.get_untracked().as_str()).is_equal_to(Some("general"));

        tabs.set_selected_key
            .run(leptonic::hooks::collections::Key::from("contact"));
        ctx.select_tab(&extras, TabId::from("history"));
        // Tabs outside of a group are ignored.
        ctx.select_tab(&details, TabId::from("history"));

        assert_that!(ctx.selected_tab(&details)).is_equal_to(Some(TabId::from("contact")));
        assert_that!(ctx.selected_tab(&extras)).is_equal_to(Some(TabId::from("history")));
        assert_that!(tabs.selected_key.get_untracked().as_str()).is_equal_to(Some("contact"));
    });
}

#[test]
fn saves_notify_the_outcomes_they_are_asked_to() {
    let cases = [
        (
            "reject",
            CrudSaveNotifications::All,
            Some(CrudNotificationKind::Error),
        ),
        (
            "reject",
            CrudSaveNotifications::Failures,
            Some(CrudNotificationKind::Error),
        ),
        ("reject", CrudSaveNotifications::None, None),
        (
            "accept",
            CrudSaveNotifications::All,
            Some(CrudNotificationKind::Success),
        ),
        ("accept", CrudSaveNotifications::Failures, None),
    ];
    for (name, notifications, expected) in cases {
        let server = FakeServer::with_items(1);
        let recorded = Arc::new(Mutex::new(Vec::<CrudNotification>::new()));
        let failures = Arc::new(Mutex::new(0));
        with_crud_manager(|| {
            let sink = recorded.clone();
            provide_crud_notifier(CrudNotifier::new(move |notification| {
                sink.lock().expect("lock").push(notification);
            }));
            provide_crud_instance(instance_input("items", config(&server)));

            let failed = failures.clone();
            let edit = use_crud_edit_form(UseCrudEditFormInput {
                on_save_failed: Some(Callback::new(move |_| *failed.lock().expect("lock") += 1)),
                notifications,
                ..edit_input(item(1).id())
            });
            settle();
            edit.form.set_field(
                DynUpdateField::from(ItemField::Name),
                Ok(Value::String(name.to_owned())),
            );
            edit.save.run(CrudSaveFollowUp::Stay);
            settle();
        });

        let kinds = recorded
            .lock()
            .expect("lock")
            .iter()
            .map(|notification| notification.kind)
            .collect::<Vec<_>>();
        assert_that!(kinds).is_equal_to(expected.into_iter().collect::<Vec<_>>());
        // The application's callback runs whether or not a failure is notified.
        let expected_failures = usize::from(name == "reject");
        assert_that!(*failures.lock().expect("lock")).is_equal_to(expected_failures);
    }
}

#[test]
fn edit_forms_report_their_status_and_save_violations() {
    let server = FakeServer::with_items(1);
    with_instance(&server, |_ctx| {
        let edit = use_crud_edit_form(edit_input(item(1).id()));
        assert_that!(edit.status.get_untracked()).is_equal_to(CrudEntityLoadStatus::Loading);
        settle();
        assert_that!(edit.status.get_untracked()).is_equal_to(CrudEntityLoadStatus::Ready);
        assert_that!(edit.violations.get_untracked().is_none()).is_true();

        edit.form.set_field(
            DynUpdateField::from(ItemField::Name),
            Ok(Value::String("renamed".to_owned())),
        );
        edit.save.run(CrudSaveFollowUp::Stay);
        settle();
        assert_that!(edit.violations.get_untracked().is_some()).is_true();
    });

    let empty = FakeServer::with_items(0);
    with_instance(&empty, |_ctx| {
        let read = use_crud_read(item(1).id());
        settle();
        assert_that!(read.status.get_untracked()).is_equal_to(CrudEntityLoadStatus::NotFound);
    });
}

/// Deletes one item against a server that answers the deletion with `status` and `body`, and
/// returns the kinds of the resulting notifications.
fn notifications_of_failed_deletion(
    status: u16,
    body: serde_json::Value,
) -> Vec<CrudNotificationKind> {
    let server = Arc::new(FakeServer {
        http: CrudTestServer::new(move |request| match request.operation.as_str() {
            "delete-by-id" => CrudTestResponse::status(status, body.clone()),
            other => panic!("unexpected operation {other}"),
        }),
    });
    let notifications = Arc::new(Mutex::new(Vec::<CrudNotification>::new()));
    with_crud_manager(|| {
        let recorded = notifications.clone();
        provide_crud_notifier(CrudNotifier::new(move |notification| {
            recorded.lock().expect("lock").push(notification);
        }));
        provide_crud_instance(instance_input("items", config(&server)));
        let deletion = crate::hooks::delete::use_crud_delete();
        deletion.request(item(1));
        deletion.confirm();
        settle();
        assert_that!(deletion.pending.get_untracked().is_none()).is_true();
    });

    notifications
        .lock()
        .expect("lock")
        .iter()
        .map(|notification| notification.kind)
        .collect()
}

#[test]
fn forbidden_deletions_are_reported_as_warnings() {
    let kinds = notifications_of_failed_deletion(
        403,
        serde_json::json!({
            "wire_format_version": 1,
            "error": { "kind": "forbidden", "message": "not allowed" },
        }),
    );
    assert_that!(kinds).is_equal_to(vec![CrudNotificationKind::Warning]);
}

#[test]
fn deletions_blocked_by_critical_validation_errors_are_reported_as_warnings() {
    let kinds = notifications_of_failed_deletion(
        422,
        serde_json::json!({
            "wire_format_version": 1,
            "error": {
                "kind": "critical_validation_errors",
                "message": "Critical validation errors prevent the operation.",
                "violations": {
                    "general": null,
                    "create": null,
                    "by_entity": [{
                        "id": [["id", { "I64": 1 }]],
                        "violations": [{ "severity": "critical", "message": "Captain." }],
                    }],
                },
            },
        }),
    );
    assert_that!(kinds).is_equal_to(vec![CrudNotificationKind::Warning]);
}

#[test]
fn deletions_stay_within_the_base_condition_of_the_rest_data_source() {
    let server = FakeServer::with_items(3);
    let mut base_condition = crudkit_core::condition::Condition::all();
    base_condition.push_elements(vec![crudkit_core::condition::ConditionElement::Clause(
        crudkit_core::condition::ConditionClause {
            column_name: "name".to_owned(),
            operator: crudkit_core::condition::Operator::Equal,
            value: crudkit_core::condition::ConditionClauseValue::String("scoped".to_owned()),
        },
    )]);
    with_crud_manager(|| {
        let data_source = CrudRestDataSource::for_resource::<CrudItemResource>(
            "http://test.local/api".to_owned(),
            server.http.executor(),
        )
        .with_base_condition(base_condition);
        let config = CrudInstanceConfig {
            data_source: Arc::new(data_source),
            ..config(&server)
        };
        provide_crud_instance(instance_input("items", config));
        let deletion = crate::hooks::delete::use_crud_delete();
        deletion.request(item(2));
        deletion.confirm();
        deletion.request_many(vec![item(1), item(3)]);
        deletion.confirm_many();
        settle();
    });

    let single = server
        .requests_to("delete-by-id")
        .pop()
        .expect("a delete-by-id request");
    assert_that!(single["condition"].to_string()).contains("scoped");
    let many = server
        .requests_to("delete-many")
        .pop()
        .expect("a delete-many request");
    let condition = many["condition"].to_string();
    assert_that!(condition.as_str()).contains("scoped");
    assert_that!(condition.as_str()).contains("Any");
}

#[test]
fn leaving_a_view_asks_about_drafts_of_instances_nested_in_it() {
    let server = FakeServer::with_items(1);
    with_crud_manager(|| {
        let parent = provide_crud_instance(instance_input("parents", config(&server)));
        let edited = CrudView::edit(item(1).id());
        parent.navigation.navigate(edited.clone());

        // The parent's edit view, which renders a nested instance holding a draft.
        let view_owner = Owner::new();
        view_owner.with(|| {
            provide_view_context(&parent, parent.navigation);
            let view_navigation = use_crud_navigation();
            // Kept alive like a mounted component; dropping an owner cleans it up.
            let nested_owner = Owner::new();
            nested_owner.with(|| {
                provide_crud_instance(instance_input("children", config(&server)));
                let create = use_crud_create_form(UseCrudCreateFormInput::default());
                create.form.set_field(
                    DynCreateField::from(CreateItemField::Name),
                    Ok(Value::String("draft".to_owned())),
                );
            });

            view_navigation.return_from_current();
            assert_that!(parent.navigation.current().get_untracked()).is_equal_to(edited);
            let confirmation = use_crud_leave_confirmation();
            assert_that!(confirmation.is_pending.get_untracked()).is_true();

            confirmation.accept();
            assert_that!(parent.navigation.current().get_untracked())
                .is_equal_to(CrudView::table());
        });
    });
}

#[test]
fn only_deleting_the_shown_entity_returns_from_the_view() {
    let server = FakeServer::with_items(3);
    with_instance(&server, |ctx| {
        let returns = Arc::new(AtomicUsize::new(0));
        let counted = returns.clone();
        ctx.navigation.return_with(move || {
            counted.fetch_add(1, Ordering::SeqCst);
        });
        let deletion = use_crud_delete();

        // Deleting a listed entity keeps the list and reloads it.
        let reload = ctx.reload.get_untracked();
        deletion.request(item(2));
        deletion.confirm();
        settle();
        assert_that!(returns.load(Ordering::SeqCst)).is_equal_to(0);
        assert_that!(ctx.reload.get_untracked()).is_not_equal_to(reload);

        ctx.navigation.navigate(CrudView::edit(item(3).id()));
        deletion.request(item(3));
        deletion.confirm();
        settle();
        assert_that!(returns.load(Ordering::SeqCst)).is_equal_to(1);
    });
}

#[test]
fn forms_follow_the_navigation_of_their_view() {
    let server = FakeServer::with_items(0);
    with_instance(&server, |ctx| {
        let drawer = ctx.navigation.child(CrudView::create());
        let returns = Arc::new(AtomicUsize::new(0));
        let counted = returns.clone();
        drawer.return_with(move || {
            counted.fetch_add(1, Ordering::SeqCst);
        });

        Owner::new().with(|| {
            // As the outlet does for a rendered view.
            crate::instance::provide_view_context(&ctx, drawer);
            let create = use_crud_create_form(UseCrudCreateFormInput::default());
            create.form.set_field(
                DynCreateField::from(CreateItemField::Name),
                Ok(Value::String("new".to_owned())),
            );

            // The draft guards the view's navigation.
            drawer.return_from_current();
            assert_that!(returns.load(Ordering::SeqCst)).is_equal_to(0);
            use_crud_leave_confirmation().cancel();

            // Saving returns on the view's navigation.
            create.save.run(CrudSaveFollowUp::Return);
            settle();
            assert_that!(returns.load(Ordering::SeqCst)).is_equal_to(1);
            assert_that!(ctx.navigation.current().get_untracked()).is_equal_to(CrudView::table());
        });
    });
}

#[test]
fn row_fields_follow_their_entity() {
    let server = FakeServer::with_items(1);
    with_instance(&server, |_ctx| {
        let entity = RwSignal::new(item(1));
        let fields = use_crud_row_fields(entity);
        let name = DynReadField::from(ReadItemField::Name);
        assert_that!(
            fields
                .value(&name)
                .and_then(|value| value.as_string().cloned())
        )
        .is_equal_to(Some("item 1".to_owned()));

        entity.set(DynReadModel::from(ReadItem {
            name: "renamed".to_owned(),
            ..ReadItem::numbered(1)
        }));
        settle();

        assert_that!(
            fields
                .value(&name)
                .and_then(|value| value.as_string().cloned())
        )
        .is_equal_to(Some("renamed".to_owned()));
    });
}

#[test]
fn rows_are_keyed_by_their_id_values() {
    assert_that!(crate::hooks::table::row_key(&item(4)).as_str()).is_equal_to(Some("4"));
}

#[test]
fn creating_another_from_a_create_form_starts_a_fresh_draft_in_place() {
    let server = FakeServer::with_items(0);
    with_instance(&server, |ctx| {
        let create = use_crud_create_form(UseCrudCreateFormInput::default());
        let name = DynCreateField::from(CreateItemField::Name);
        create
            .form
            .set_field(name.clone(), Ok(Value::String("first".to_owned())));
        let view_before = ctx.navigation.current().get_untracked();
        create.save.run(CrudSaveFollowUp::CreateAnother);
        settle();

        assert_that!(server.requests_to("create-one").len()).is_equal_to(1);
        assert_that!(create.form.is_dirty.get_untracked()).is_false();
        assert_that!(ctx.navigation.current().get_untracked()).is_equal_to(view_before);
    });
}

#[test]
fn reloading_keeps_unsaved_changes_of_the_entity() {
    let server = FakeServer::with_items(1);
    with_instance(&server, |ctx| {
        let edit = use_crud_edit_form(edit_input(item(1).id()));
        settle();
        let name = DynUpdateField::from(ItemField::Name);
        edit.form
            .set_field(name.clone(), Ok(Value::String("unsaved".to_owned())));

        ctx.reload();
        settle();

        assert_that!(server.requests_to("read-one").len()).is_equal_to(2);
        assert_that!(edit.form.is_dirty.get_untracked()).is_true();
        assert_that!(
            edit.form
                .field(&name)
                .and_then(|field| field.get_untracked().as_string().cloned())
        )
        .is_equal_to(Some("unsaved".to_owned()));
    });
}

#[test]
fn deletions_finishing_after_their_instance_unmounted_are_still_reported() {
    let server = FakeServer::with_items(3);
    let notifications = Arc::new(Mutex::new(Vec::<CrudNotification>::new()));
    with_crud_manager(|| {
        let recorded = notifications.clone();
        provide_crud_notifier(CrudNotifier::new(move |notification| {
            recorded.lock().expect("lock").push(notification);
        }));
        let instance = Owner::new();
        instance.with(|| {
            provide_crud_instance(instance_input("items", config(&server)));
            let deletion = crate::hooks::delete::use_crud_delete();
            deletion.request(item(2));
            deletion.confirm();
            deletion.request_many(Arc::new(vec![item(1)]));
            deletion.confirm_many();
        });
        // The instance unmounts while both requests are in flight.
        drop(instance);
        settle();
    });

    assert_that!(server.requests_to("delete-by-id").len()).is_equal_to(1);
    assert_that!(server.requests_to("delete-many").len()).is_equal_to(1);
    assert_that!(notifications.lock().expect("lock").len()).is_equal_to(2);
}

/// Returns the IDs of the listed items.
fn listed_ids(list: &CrudListState) -> Vec<i64> {
    list.rows
        .get_untracked()
        .loaded()
        .map(|rows| {
            rows.iter()
                .map(|row| row.downcast_ref::<ReadItem>().id)
                .collect()
        })
        .unwrap_or_default()
}

/// Returns the cursor of the last list request that `source` received.
fn last_cursor(source: &MemorySource) -> Option<String> {
    source
        .list_requests()
        .pop()
        .expect("a list request")
        .cursor
        .map(|cursor| cursor.as_str().to_owned())
}

#[test]
fn lists_paged_by_cursor_offer_only_the_previous_and_the_next_page() {
    let source = MemorySource::with_items(5, MemoryPaging::Cursor);
    with_config(source.config(), |_ctx| {
        let list = use_crud_list_state();
        settle();
        assert_that!(listed_ids(&list)).is_equal_to(vec![1, 2]);
        assert_that!(list.item_count.get_untracked()).is_none();
        assert_that!(list.pagination.page_count.get_untracked()).is_equal_to(0);
        assert_that!(list.pagination.page_options.get_untracked().options).is_empty();
        assert_that!(list.pagination.range()).is_none();
        assert_that!(list.pagination.has_previous()).is_false();
        assert_that!(list.pagination.has_next()).is_true();

        list.pagination.next();
        settle();
        assert_that!(listed_ids(&list)).is_equal_to(vec![3, 4]);
        assert_that!(last_cursor(&source)).is_equal_to(Some("2".to_owned()));

        list.pagination.next();
        settle();
        assert_that!(listed_ids(&list)).is_equal_to(vec![5]);
        assert_that!(list.pagination.has_next()).is_false();
        list.pagination.next();
        assert_that!(list.pagination.page.get_untracked()).is_equal_to(PageNr(3));

        // The list opens the pages it visited with their cursors again.
        list.pagination.previous();
        settle();
        assert_that!(listed_ids(&list)).is_equal_to(vec![3, 4]);
        assert_that!(last_cursor(&source)).is_equal_to(Some("2".to_owned()));
        list.pagination.previous();
        settle();
        assert_that!(listed_ids(&list)).is_equal_to(vec![1, 2]);
        assert_that!(last_cursor(&source)).is_none();
    });
}

#[test]
fn lists_paged_by_cursor_start_over_when_their_ordering_or_page_size_changes() {
    let source = MemorySource::with_items(5, MemoryPaging::Cursor);
    with_config(source.config(), |_ctx| {
        let list = use_crud_list_state();
        settle();
        list.pagination.next();
        settle();

        list.ordering
            .toggle(DynReadField::from(ReadItemField::Name), false);
        settle();
        assert_that!(list.pagination.page.get_untracked()).is_equal_to(PageNr::first());
        let request = source.list_requests().pop().expect("a list request");
        assert_that!(request.cursor).is_none();
        assert_that!(request.order_by.len()).is_equal_to(1);

        list.pagination.next();
        settle();
        list.pagination.set_items_per_page(ItemsPerPage(3));
        settle();
        assert_that!(list.pagination.page.get_untracked()).is_equal_to(PageNr::first());
        assert_that!(last_cursor(&source)).is_none();
        assert_that!(listed_ids(&list)).is_equal_to(vec![1, 2, 3]);
    });
}

#[test]
fn lists_paged_by_offset_keep_their_page_when_their_ordering_changes() {
    let source = MemorySource::with_items(5, MemoryPaging::Offset);
    with_config(source.config(), |_ctx| {
        let list = use_crud_list_state();
        settle();
        assert_that!(list.item_count.get_untracked()).is_equal_to(Some(5));
        assert_that!(list.pagination.page_options.get_untracked().options.len()).is_equal_to(3);

        list.pagination.set_page(PageNr(2));
        list.ordering
            .toggle(DynReadField::from(ReadItemField::Name), false);
        settle();
        assert_that!(list.pagination.page.get_untracked()).is_equal_to(PageNr(2));
        let request = source.list_requests().pop().expect("a list request");
        assert_that!(request.offset).is_equal_to(2);
        assert_that!(request.cursor).is_none();
    });
}

#[test]
fn lists_paged_by_cursor_step_back_from_pages_they_cannot_show() {
    // A configured page beyond the first cannot be reached without the cursors before it.
    let source = MemorySource::with_items(3, MemoryPaging::Cursor);
    let config = CrudInstanceConfig {
        page_nr: PageNr(2),
        ..source.config()
    };
    with_config(config, |_ctx| {
        let list = use_crud_list_state();
        settle();
        assert_that!(list.pagination.page.get_untracked()).is_equal_to(PageNr::first());
        assert_that!(listed_ids(&list)).is_equal_to(vec![1, 2]);

        // A page left empty by a deletion is left for the page before it.
        list.pagination.next();
        settle();
        assert_that!(listed_ids(&list)).is_equal_to(vec![3]);
        let deletion = use_crud_delete();
        deletion.request(item(3));
        deletion.confirm();
        settle();
        assert_that!(list.pagination.page.get_untracked()).is_equal_to(PageNr::first());
        assert_that!(listed_ids(&list)).is_equal_to(vec![1, 2]);
        assert_that!(list.pagination.has_next()).is_false();
    });
}

#[test]
fn lists_order_only_by_fields_their_data_source_orders_by() {
    let source = MemorySource::with_items(3, MemoryPaging::Offset);
    let name = DynReadField::from(ReadItemField::Name);
    let id = DynReadField::from(ReadItemField::Id);
    let mut order_by = IndexMap::new();
    order_by.insert(id.clone(), Order::Desc);
    order_by.insert(name.clone(), Order::Desc);
    let config = CrudInstanceConfig {
        order_by,
        ..source.config()
    };
    with_config(config, |_ctx| {
        let list = use_crud_list_state();
        settle();
        assert_that!(list.ordering.can_order_by(&name)).is_true();
        assert_that!(list.ordering.can_order_by(&id)).is_false();
        // The configured ordering by ID is left out.
        let expected = IndexMap::from([(name.clone(), Order::Desc)]);
        assert_that!(list.ordering.order_by().get_untracked()).is_equal_to(expected.clone());

        list.ordering.toggle(id.clone(), false);
        list.ordering.set(id.clone(), Order::Asc);
        settle();
        assert_that!(list.ordering.order_by().get_untracked()).is_equal_to(expected.clone());
        let request = source.list_requests().pop().expect("a list request");
        assert_that!(request.order_by).is_equal_to(expected);
        assert_that!(listed_ids(&list)).is_equal_to(vec![3, 2]);
    });
}

#[test]
fn every_operation_of_a_nested_instance_happens_below_its_parents() {
    let source = MemorySource::with_items(3, MemoryPaging::Offset);
    with_crud_manager(|| {
        let parent = provide_crud_instance(instance_input("parents", source.config()));
        parent.navigation.navigate(CrudView::edit(item(1).id()));

        Owner::new().with(|| {
            // E.g. the owner from the application's route.
            let config = CrudInstanceConfig {
                scope: Signal::stored(
                    CrudScope::default().with_parent("owner_id", IdValue::I64(7)),
                ),
                ..source.config()
            };
            provide_crud_instance(ProvideCrudInstanceInput {
                parent: Some(CrudParentConfig {
                    name: "parents",
                    referenced_field: "id".into(),
                    referencing_field: "item_id".into(),
                }),
                ..instance_input("children", config)
            });
            let _list = use_crud_list_state();
            // The create model has no field referring to a parent, as with sources that put the
            // parents into a path.
            let create = use_crud_create_form(UseCrudCreateFormInput::default());
            create.form.set_field(
                DynCreateField::from(CreateItemField::Name),
                Ok(Value::String("new".to_owned())),
            );
            create.save.run(CrudSaveFollowUp::Stay);
            let edit = use_crud_edit_form(edit_input(item(2).id()));
            settle();
            edit.form.set_field(
                DynUpdateField::from(ItemField::Name),
                Ok(Value::String("renamed".to_owned())),
            );
            edit.save.run(CrudSaveFollowUp::Stay);
            let deletion = use_crud_delete();
            deletion.request(item(3));
            deletion.confirm();
            settle();
        });
    });

    let below_parents = CrudScope::default()
        .with_parent("owner_id", IdValue::I64(7))
        .with_parent("item_id", IdValue::I64(1));
    let operations = source.operations();
    let kinds = |scope: &CrudScope| {
        operations
            .iter()
            .filter(|(_, received)| received == scope)
            .map(|(operation, _)| operation.kind())
            .collect::<std::collections::BTreeSet<_>>()
    };
    assert_that!(kinds(&below_parents).into_iter().collect::<Vec<_>>())
        .is_equal_to(vec!["count", "create", "delete", "list", "read", "update"]);
    // Only the parent instance's own list ran without parents, and it listed nothing here.
    assert_that!(kinds(&CrudScope::default())).is_empty();
}

#[test]
fn new_entities_refer_to_the_parents_of_their_scope() {
    let source = MemorySource::with_items(0, MemoryPaging::Offset);
    let config = CrudInstanceConfig {
        // The item's name stands in for a field that refers to a parent.
        scope: Signal::stored(
            CrudScope::default().with_parent("name", IdValue::String("scoped".to_owned())),
        ),
        ..source.config()
    };
    with_config(config, |_ctx| {
        let create = use_crud_create_form(UseCrudCreateFormInput::default());
        assert_that!(
            create
                .form
                .field(&DynCreateField::from(CreateItemField::Name))
                .and_then(|field| field.get_untracked().as_string().cloned())
        )
        .is_equal_to(Some("scoped".to_owned()));
    });
}

#[test]
fn data_sources_without_mass_deletion_delete_one_entity_after_the_other() {
    let source = MemorySource::with_items(3, MemoryPaging::Offset);
    let notifications = Arc::new(Mutex::new(Vec::<CrudNotification>::new()));
    with_crud_manager(|| {
        let recorded = notifications.clone();
        provide_crud_notifier(CrudNotifier::new(move |notification| {
            recorded.lock().expect("lock").push(notification);
        }));
        provide_crud_instance(instance_input("items", source.config()));
        let deletion = use_crud_delete();
        deletion.request_many(vec![item(1), item(3)]);
        deletion.confirm_many();
        settle();
    });

    assert_that!(source.ids()).is_equal_to(vec![2]);
    let message = notifications
        .lock()
        .expect("lock")
        .iter()
        .map(|notification| (notification.kind, notification.message.clone()))
        .collect::<Vec<_>>();
    assert_that!(message).is_equal_to(vec![(
        CrudNotificationKind::Success,
        "2 entries deleted.".to_owned(),
    )]);
}

#[test]
fn rest_data_sources_keep_nested_instances_below_their_parent_by_condition() {
    let server = FakeServer::with_items(2);
    with_crud_manager(|| {
        let parent = provide_crud_instance(instance_input("parents", config(&server)));
        parent.navigation.navigate(CrudView::edit(item(1).id()));
        Owner::new().with(|| {
            provide_crud_instance(ProvideCrudInstanceInput {
                parent: Some(CrudParentConfig {
                    name: "parents",
                    referenced_field: "id".into(),
                    referencing_field: "owner_id".into(),
                }),
                ..instance_input("children", config(&server))
            });
            let _list = use_crud_list_state();
            settle();
        });
    });

    let condition = |operation: &str| {
        server
            .requests_to(operation)
            .pop()
            .map(|request| request["condition"].to_string())
            .unwrap_or_default()
    };
    assert_that!(condition("read-many").as_str()).contains("owner_id");
    assert_that!(condition("read-count").as_str()).contains("owner_id");
}

#[test]
fn lists_start_over_below_other_parents_while_they_stay_mounted() {
    for paging in [MemoryPaging::Cursor, MemoryPaging::Offset] {
        let source = MemorySource::with_items(5, paging);
        // E.g. the parameters of the application's route.
        let project = RwSignal::new(1);
        let config = CrudInstanceConfig {
            scope: Signal::derive(move || {
                CrudScope::default().with_parent("project_id", IdValue::I64(project.get()))
            }),
            ..source.config()
        };
        with_config(config, |_ctx| {
            let list = use_crud_list_state();
            settle();
            list.pagination.next();
            settle();
            assert_that!(list.pagination.page.get_untracked()).is_equal_to(PageNr(2));

            project.set(2);
            settle();
            assert_that!(list.pagination.page.get_untracked()).is_equal_to(PageNr::first());
            assert_that!(listed_ids(&list)).is_equal_to(vec![1, 2]);
            let (request, scope) = source
                .operations()
                .into_iter()
                .rev()
                .find_map(|(operation, scope)| match operation {
                    MemoryOperation::List(request) => Some((request, scope)),
                    _ => None,
                })
                .expect("a list request");
            assert_that!(scope.parent("project_id")).is_equal_to(Some(&IdValue::I64(2)));
            assert_that!(request.offset).is_equal_to(0);
            assert_that!(request.cursor).is_none();
        });
    }
}

#[test]
fn updates_name_the_version_they_start_from() {
    let source = MemorySource::with_items(1, MemoryPaging::Offset);
    with_config(source.config(), |_ctx| {
        let edit = use_crud_edit_form(edit_input(item(1).id()));
        settle();
        let name = DynUpdateField::from(ItemField::Name);
        for renamed in ["first", "second"] {
            edit.form
                .set_field(name.clone(), Ok(Value::String(renamed.to_owned())));
            edit.save.run(CrudSaveFollowUp::Stay);
            settle();
        }
    });

    let updates = source
        .operations()
        .into_iter()
        .filter_map(|(operation, _)| match operation {
            MemoryOperation::Update(request) => Some(request),
            _ => None,
        })
        .map(|request| {
            let name = |model: DynUpdateModel| model.downcast::<Item>().name;
            (name(request.baseline), name(request.entity))
        })
        .collect::<Vec<_>>();
    // The second change starts from the version that the first one saved.
    assert_that!(updates).is_equal_to(vec![
        ("item 1".to_owned(), "first".to_owned()),
        ("first".to_owned(), "second".to_owned()),
    ]);
}

#[test]
fn deletions_name_the_entity_as_the_user_saw_it() {
    let source = MemorySource::with_items(2, MemoryPaging::Offset);
    with_config(source.config(), |_ctx| {
        let deletion = use_crud_delete();
        deletion.request(item(2));
        deletion.confirm();
        settle();
    });

    let deleted = source
        .operations()
        .into_iter()
        .find_map(|(operation, _)| match operation {
            MemoryOperation::Delete(request) => Some(request),
            _ => None,
        })
        .expect("a deletion");
    assert_that!(deleted.id).is_equal_to(item(2).id());
    assert_that!(deleted.entity).is_equal_to(DynReadOrUpdateModel::from(item(2)));
}

#[test]
fn rest_lists_count_again_only_when_they_reload() {
    let server = FakeServer::with_items(5);
    with_instance(&server, |ctx| {
        let list = use_crud_list_state();
        settle();
        list.pagination.next();
        list.ordering
            .toggle(DynReadField::from(ReadItemField::Name), false);
        settle();
        assert_that!(server.requests_to("read-many").len()).is_equal_to(2);
        assert_that!(server.requests_to("read-count").len()).is_equal_to(1);

        ctx.reload();
        settle();
        assert_that!(server.requests_to("read-count").len()).is_equal_to(2);
    });
}

#[test]
fn rest_lists_show_their_page_when_counting_fails() {
    let items = FakeServer::with_items(3);
    let server = Arc::new(FakeServer {
        http: CrudTestServer::new(move |request| match request.operation.as_str() {
            "read-count" => CrudTestResponse::status(
                500,
                serde_json::json!({
                    "wire_format_version": 1,
                    "error": { "kind": "internal_server_error", "message": "count failed" },
                }),
            ),
            _ => items.http.answer(request),
        }),
    });
    with_instance(&server, |_ctx| {
        let list = use_crud_list_state();
        settle();
        assert_that!(listed_ids(&list)).is_equal_to(vec![1, 2]);
        assert_that!(list.status.get_untracked()).is_equal_to(CrudListLoadStatus::Ready);
        assert_that!(list.item_count.get_untracked()).is_none();
    });
}

/// Provides a notifier in the current owner that records every notification in `recorded`.
fn record_notifications(recorded: &Arc<Mutex<Vec<CrudNotification>>>) {
    let sink = recorded.clone();
    provide_crud_notifier(CrudNotifier::new(move |notification| {
        sink.lock().expect("lock").push(notification);
    }));
}

/// Returns the kind and message of each notification in `recorded`, oldest first.
fn kinds_and_messages(
    recorded: &Mutex<Vec<CrudNotification>>,
) -> Vec<(CrudNotificationKind, String)> {
    recorded
        .lock()
        .expect("lock")
        .iter()
        .map(|notification| (notification.kind, notification.message.clone()))
        .collect()
}

/// Returns the English default of `text`.
fn english(text: CrudText<'_>) -> String {
    text.english().into_owned()
}

/// Returns the update requests that `source` received, oldest first.
fn update_requests(source: &MemorySource) -> Vec<CrudUpdateRequest> {
    source
        .operations()
        .into_iter()
        .filter_map(|(operation, _)| match operation {
            MemoryOperation::Update(request) => Some(request),
            _ => None,
        })
        .collect()
}

/// Returns the text that `form` holds for `field`.
fn text_of(form: &CrudFormState<DynUpdateField>, field: ItemField) -> Option<String> {
    form.field(&DynUpdateField::from(field))
        .and_then(|value| value.get_untracked().as_string().cloned())
}

/// Returns the item `id` of `source` as a listed entity, with its version.
fn listed(source: &MemorySource, id: i64) -> DynReadModel {
    DynReadModel::from(source.item(id).expect("the item exists"))
}

#[test]
fn changes_send_what_changed_and_the_version_they_are_based_on() {
    let source = MemorySource::with_items(1, MemoryPaging::Offset);
    with_config(source.config(), |_ctx| {
        let edit = use_crud_edit_form(edit_input(item(1).id()));
        settle();
        for renamed in ["first", "second"] {
            edit.form.set_field(
                DynUpdateField::from(ItemField::Name),
                Ok(Value::String(renamed.to_owned())),
            );
            edit.save.run(CrudSaveFollowUp::Stay);
            settle();
        }
    });

    let based_on = update_requests(&source)
        .iter()
        .map(|request| {
            (
                request.version().map(|version| version.as_str().to_owned()),
                request.merge_patch().expect("a merge patch"),
            )
        })
        .collect::<Vec<_>>();
    // The second change is based on the version that the first one saved.
    assert_that!(based_on).is_equal_to(vec![
        (Some("1".to_owned()), serde_json::json!({ "name": "first" })),
        (
            Some("2".to_owned()),
            serde_json::json!({ "name": "second" }),
        ),
    ]);
    let stored = source.item(1).expect("the item");
    assert_that!(stored.version).is_equal_to(Some(CrudVersion::new("3")));
}

#[test]
fn outdated_changes_keep_the_input_and_apply_it_to_the_current_state() {
    let source = MemorySource::with_items(1, MemoryPaging::Offset);
    let notifications = Arc::new(Mutex::new(Vec::new()));
    with_crud_manager(|| {
        record_notifications(&notifications);
        provide_crud_instance(instance_input("items", source.config()));
        let edit = use_crud_edit_form(edit_input(item(1).id()));
        settle();
        edit.form.set_field(
            DynUpdateField::from(ItemField::Name),
            Ok(Value::String("mine".to_owned())),
        );
        source.change_elsewhere(1, |item| "theirs".clone_into(&mut item.note));

        edit.save.run(CrudSaveFollowUp::Stay);
        settle();
        assert_that!(matches!(
            edit.form.save_error.get_untracked(),
            Some(RequestError::Outdated(_))
        ))
        .is_true();
        assert_that!(edit.form.is_dirty.get_untracked()).is_true();
        assert_that!(text_of(&edit.form, ItemField::Name)).is_equal_to(Some("mine".to_owned()));

        // The current state shows the other change, and the input stays.
        edit.reload.run(());
        settle();
        assert_that!(edit.form.save_error.get_untracked()).is_none();
        assert_that!(text_of(&edit.form, ItemField::Name)).is_equal_to(Some("mine".to_owned()));
        assert_that!(text_of(&edit.form, ItemField::Note)).is_equal_to(Some("theirs".to_owned()));
        assert_that!(edit.form.is_dirty.get_untracked()).is_true();

        edit.save.run(CrudSaveFollowUp::Stay);
        settle();
        assert_that!(edit.form.save_error.get_untracked()).is_none();
        assert_that!(edit.form.is_dirty.get_untracked()).is_false();
    });

    // The change applied to the current version, and kept the other change.
    let retried = update_requests(&source).pop().expect("an update");
    assert_that!(retried.version()).is_equal_to(Some(&CrudVersion::new("2")));
    assert_that!(retried.merge_patch()).is_equal_to(Ok(serde_json::json!({ "name": "mine" })));
    let stored = source.item(1).expect("the item");
    assert_that!((stored.name.as_str(), stored.note.as_str())).is_equal_to(("mine", "theirs"));
    assert_that!(kinds_and_messages(&notifications)).is_equal_to(vec![
        (
            CrudNotificationKind::Warning,
            english(CrudText::FormOutdated),
        ),
        (
            CrudNotificationKind::Success,
            english(CrudText::SaveUpdated),
        ),
    ]);
}

#[test]
fn problems_reach_the_fields_they_name_until_the_user_changes_them() {
    let source = MemorySource::with_items(1, MemoryPaging::Offset);
    with_config(source.config(), |_ctx| {
        let edit = use_crud_edit_form(edit_input(item(1).id()));
        settle();
        let name = DynUpdateField::from(ItemField::Name);
        edit.form
            .set_field(name.clone(), Ok(Value::String(TAKEN_NAME.to_owned())));
        edit.save.run(CrudSaveFollowUp::Stay);
        settle();

        assert_that!(edit.form.field_error(name.clone()).get_untracked())
            .is_equal_to(Some("Another item has this name.".to_owned()));
        assert_that!(matches!(
            edit.form.save_error.get_untracked(),
            Some(RequestError::InvalidFields { .. })
        ))
        .is_true();
        let problem_fields = move || {
            edit.form
                .problems
                .get_untracked()
                .into_iter()
                .map(|problem| problem.field)
                .collect::<Vec<_>>()
        };
        assert_that!(problem_fields()).is_equal_to(vec!["name".to_owned(), "secret".to_owned()]);

        // Changing the field resolves its problem, but not the others.
        edit.form
            .set_field(name.clone(), Ok(Value::String("free".to_owned())));
        assert_that!(edit.form.field_error(name).get_untracked()).is_none();
        assert_that!(problem_fields()).is_equal_to(vec!["secret".to_owned()]);

        edit.save.run(CrudSaveFollowUp::Stay);
        settle();
        assert_that!(edit.form.save_error.get_untracked()).is_none();
        assert_that!(edit.form.problems.get_untracked()).is_empty();
    });
}

#[test]
fn problems_of_new_entities_reach_their_fields() {
    let source = MemorySource::with_items(0, MemoryPaging::Offset);
    with_config(source.config(), |_ctx| {
        let create = use_crud_create_form(UseCrudCreateFormInput::default());
        let name = DynCreateField::from(CreateItemField::Name);
        create
            .form
            .set_field(name.clone(), Ok(Value::String(TAKEN_NAME.to_owned())));
        create.save.run(CrudSaveFollowUp::Stay);
        settle();

        assert_that!(create.form.field_error(name).get_untracked())
            .is_equal_to(Some("Another item has this name.".to_owned()));
        // The draft stays, so that the user can correct it.
        assert_that!(create.form.is_dirty.get_untracked()).is_true();
    });
}

#[test]
fn accepted_changes_continue_from_the_state_they_return() {
    let source = MemorySource::with_items(1, MemoryPaging::Offset);
    source.accept_changes();
    let notifications = Arc::new(Mutex::new(Vec::new()));
    let completions = Arc::new(Mutex::new(Vec::new()));
    with_crud_manager(|| {
        record_notifications(&notifications);
        provide_crud_instance(instance_input("items", source.config()));
        let saved = completions.clone();
        let edit = use_crud_edit_form(UseCrudEditFormInput {
            on_saved: Some(Callback::new(move |outcome: CrudSaved| {
                saved.lock().expect("lock").push(outcome.completion);
            })),
            ..edit_input(item(1).id())
        });
        settle();
        for renamed in ["first", "second"] {
            edit.form.set_field(
                DynUpdateField::from(ItemField::Name),
                Ok(Value::String(renamed.to_owned())),
            );
            edit.save.run(CrudSaveFollowUp::Stay);
            settle();
            assert_that!(edit.form.is_dirty.get_untracked()).is_false();
            assert_that!(edit.form.save_error.get_untracked()).is_none();
        }
    });

    // The second change is based on the version that the first one returned.
    let versions = update_requests(&source)
        .iter()
        .map(|request| request.version().cloned())
        .collect::<Vec<_>>();
    assert_that!(versions).is_equal_to(vec![
        Some(CrudVersion::new("1")),
        Some(CrudVersion::new("2")),
    ]);
    assert_that!(completions.lock().expect("lock").clone())
        .is_equal_to(vec![CrudCompletion::Accepted; 2]);
    let accepted = english(CrudText::SaveUpdateAccepted);
    assert_that!(kinds_and_messages(&notifications)).is_equal_to(vec![
        (
            CrudNotificationKind::Success,
            accepted
        );
        2
    ]);
}

#[test]
fn accepted_and_outdated_deletions_are_reported_as_such() {
    let source = MemorySource::with_items(3, MemoryPaging::Offset);
    source.accept_changes();
    let notifications = Arc::new(Mutex::new(Vec::new()));
    let mut reloaded = false;
    with_crud_manager(|| {
        record_notifications(&notifications);
        let ctx = provide_crud_instance(instance_input("items", source.config()));
        let deletion = use_crud_delete();
        deletion.request(listed(&source, 1));
        deletion.confirm();
        settle();
        deletion.request_many(vec![listed(&source, 2), listed(&source, 3)]);
        deletion.confirm_many();
        settle();

        // Someone else changes the entity that the user is about to delete.
        let seen = listed(&source, 1);
        source.change_elsewhere(1, |item| "theirs".clone_into(&mut item.note));
        let reload = ctx.reload.get_untracked();
        deletion.request(seen);
        deletion.confirm();
        settle();
        reloaded = ctx.reload.get_untracked() != reload;
    });

    // The instance shows the current state of the entity it could not delete.
    assert_that!(reloaded).is_true();
    // Accepted deletions keep their entities while they run.
    assert_that!(source.ids()).is_equal_to(vec![1, 2, 3]);
    assert_that!(kinds_and_messages(&notifications)).is_equal_to(vec![
        (
            CrudNotificationKind::Success,
            english(CrudText::DeleteAccepted { count: 1 }),
        ),
        (
            CrudNotificationKind::Success,
            english(CrudText::DeleteAccepted { count: 2 }),
        ),
        (
            CrudNotificationKind::Warning,
            english(CrudText::DeleteOutdated),
        ),
    ]);
}

#[test]
fn deleting_an_edited_entity_is_based_on_its_last_saved_version() {
    let source = MemorySource::with_items(1, MemoryPaging::Offset);
    with_config(source.config(), |_ctx| {
        let edit = use_crud_edit_form(edit_input(item(1).id()));
        settle();
        edit.form.set_field(
            DynUpdateField::from(ItemField::Name),
            Ok(Value::String("renamed".to_owned())),
        );
        edit.save.run(CrudSaveFollowUp::Stay);
        settle();

        edit.delete.run(());
        use_crud_delete().confirm();
        settle();
    });

    let deleted = source
        .operations()
        .into_iter()
        .find_map(|(operation, _)| match operation {
            MemoryOperation::Delete(request) => Some(request),
            _ => None,
        })
        .expect("a deletion");
    assert_that!(deleted.version()).is_equal_to(Some(&CrudVersion::new("2")));
    assert_that!(source.ids()).is_empty();
}

#[test]
fn clean_forms_show_every_reloaded_version_in_place() {
    let source = MemorySource::with_items(1, MemoryPaging::Offset);
    with_config(source.config(), |ctx| {
        let edit = use_crud_edit_form(edit_input(item(1).id()));
        settle();
        let note = edit.form.field(&DynUpdateField::from(ItemField::Note));

        for change in ["first", "second"] {
            source.change_elsewhere(1, |item| change.clone_into(&mut item.note));
            ctx.reload();
            settle();
            assert_that!(text_of(&edit.form, ItemField::Note)).is_equal_to(Some(change.to_owned()));
            assert_that!(edit.form.is_dirty.get_untracked()).is_false();
        }
        // The form kept the reactive value that its fields render.
        assert_that!(note.and_then(|note| note.get_untracked().as_string().cloned()))
            .is_equal_to(Some("second".to_owned()));
    });
}

/// Returns the JSON that `form` holds for `field`.
fn json_of(form: &CrudFormState<DynUpdateField>, field: ItemField) -> Option<serde_json::Value> {
    form.field(&DynUpdateField::from(field))
        .and_then(|value| value.get_untracked().as_json().cloned())
}

/// Sets `field` of `form` to `value`, as the user would.
fn enter(form: &CrudFormState<DynUpdateField>, field: ItemField, value: Value) {
    form.set_field(DynUpdateField::from(field), Ok(value));
}

#[test]
fn changes_to_members_of_an_object_field_keep_the_changes_of_others() {
    let source = MemorySource::with_items(1, MemoryPaging::Offset);
    source.change_elsewhere(1, |item| item.tags = serde_json::json!({ "a": "1" }));
    with_config(source.config(), |_ctx| {
        let edit = use_crud_edit_form(edit_input(item(1).id()));
        settle();
        enter(
            &edit.form,
            ItemField::Tags,
            Value::Json(serde_json::json!({ "a": "1", "mine": "m" })),
        );
        source.change_elsewhere(1, |item| {
            item.tags = serde_json::json!({ "a": "1", "theirs": "t" });
        });
        edit.save.run(CrudSaveFollowUp::Stay);
        settle();

        edit.reload.run(());
        settle();
        assert_that!(json_of(&edit.form, ItemField::Tags)).is_equal_to(Some(
            serde_json::json!({ "a": "1", "mine": "m", "theirs": "t" }),
        ));
        assert_that!(edit.form.conflicts.get_untracked()).is_empty();

        edit.save.run(CrudSaveFollowUp::Stay);
        settle();
    });

    let retried = update_requests(&source).pop().expect("an update");
    assert_that!(retried.merge_patch())
        .is_equal_to(Ok(serde_json::json!({ "tags": { "mine": "m" } })));
    assert_that!(source.item(1).expect("the item").tags)
        .is_equal_to(serde_json::json!({ "a": "1", "mine": "m", "theirs": "t" }));
}

#[test]
fn members_that_both_changed_conflict_until_the_user_changes_them() {
    let source = MemorySource::with_items(1, MemoryPaging::Offset);
    source.change_elsewhere(1, |item| item.tags = serde_json::json!({ "a": "1" }));
    with_config(source.config(), |_ctx| {
        let edit = use_crud_edit_form(edit_input(item(1).id()));
        settle();
        enter(
            &edit.form,
            ItemField::Name,
            Value::String("mine".to_owned()),
        );
        enter(
            &edit.form,
            ItemField::Tags,
            Value::Json(serde_json::json!({ "a": "mine" })),
        );
        source.change_elsewhere(1, |item| {
            "theirs".clone_into(&mut item.name);
            item.tags = serde_json::json!({ "a": "theirs" });
        });
        edit.save.run(CrudSaveFollowUp::Stay);
        settle();
        edit.reload.run(());
        settle();

        // The draft keeps the user's values, and each field tells what the other value is.
        let conflicts = edit
            .form
            .conflicts
            .get_untracked()
            .into_iter()
            .map(|conflict| (conflict.field, conflict.pointer, conflict.theirs))
            .collect::<Vec<_>>();
        let theirs = Some(serde_json::json!("theirs"));
        assert_that!(conflicts).is_equal_to(vec![
            ("name".to_owned(), "/name".to_owned(), theirs.clone()),
            ("tags".to_owned(), "/tags/a".to_owned(), theirs),
        ]);
        assert_that!(text_of(&edit.form, ItemField::Name)).is_equal_to(Some("mine".to_owned()));
        assert_that!(json_of(&edit.form, ItemField::Tags))
            .is_equal_to(Some(serde_json::json!({ "a": "mine" })));
        let changed = english(CrudText::FormChangedMeanwhile { value: "theirs" });
        let error_of = |field: ItemField| {
            edit.form
                .field_error(DynUpdateField::from(field))
                .get_untracked()
        };
        assert_that!(error_of(ItemField::Name)).is_equal_to(Some(changed.clone()));
        assert_that!(error_of(ItemField::Tags)).is_equal_to(Some(format!("a: {changed}")));

        // Changing a field again settles its conflict.
        enter(
            &edit.form,
            ItemField::Name,
            Value::String("mine".to_owned()),
        );
        assert_that!(error_of(ItemField::Name)).is_none();
        assert_that!(edit.form.conflicts.get_untracked().len()).is_equal_to(1);

        edit.save.run(CrudSaveFollowUp::Stay);
        settle();
        assert_that!(edit.form.conflicts.get_untracked()).is_empty();
    });

    let saved = update_requests(&source).pop().expect("an update");
    assert_that!(saved.merge_patch()).is_equal_to(Ok(serde_json::json!({
        "name": "mine",
        "tags": { "a": "mine" },
    })));
}

#[test]
fn unreadable_input_takes_the_reloaded_value_and_keeps_its_error() {
    let source = MemorySource::with_items(1, MemoryPaging::Offset);
    with_config(source.config(), |ctx| {
        let edit = use_crud_edit_form(edit_input(item(1).id()));
        settle();
        let note = DynUpdateField::from(ItemField::Note);
        let input_error = {
            let note = note.clone();
            move || edit.form.errors.get_untracked().get(&note).cloned()
        };

        let other_note = move || {
            edit.form
                .conflicts
                .get_untracked()
                .into_iter()
                .map(|conflict| (conflict.pointer, conflict.theirs))
                .collect::<Vec<_>>()
        };

        // A form whose only input cannot be read keeps that input's error over a reload, and
        // shows the other value next to it, so that the user sees it before typing again.
        edit.form
            .set_field(note.clone(), Err("unreadable".to_owned()));
        source.change_elsewhere(1, |item| "theirs".clone_into(&mut item.note));
        ctx.reload();
        settle();
        assert_that!(input_error()).is_equal_to(Some("unreadable".to_owned()));
        assert_that!(text_of(&edit.form, ItemField::Note)).is_equal_to(Some("theirs".to_owned()));
        assert_that!(other_note()).is_equal_to(vec![(
            "/note".to_owned(),
            Some(serde_json::json!("theirs")),
        )]);
        let changed = english(CrudText::FormChangedMeanwhile { value: "theirs" });
        assert_that!(edit.form.field_error(note).get_untracked())
            .is_equal_to(Some(format!("unreadable {changed}")));

        // Next to a change, too.
        enter(
            &edit.form,
            ItemField::Name,
            Value::String("mine".to_owned()),
        );
        source.change_elsewhere(1, |item| "again".clone_into(&mut item.note));
        ctx.reload();
        settle();
        assert_that!(input_error()).is_equal_to(Some("unreadable".to_owned()));
        assert_that!(text_of(&edit.form, ItemField::Note)).is_equal_to(Some("again".to_owned()));
        assert_that!(text_of(&edit.form, ItemField::Name)).is_equal_to(Some("mine".to_owned()));
        assert_that!(other_note())
            .is_equal_to(vec![("/note".to_owned(), Some(serde_json::json!("again")))]);
    });
}

#[test]
fn typing_into_a_field_after_a_rebase_keeps_working() {
    let source = MemorySource::with_items(1, MemoryPaging::Offset);
    with_config(source.config(), |ctx| {
        let edit = use_crud_edit_form(edit_input(item(1).id()));
        settle();
        for (round, change) in ["first", "second"].into_iter().enumerate() {
            enter(
                &edit.form,
                ItemField::Name,
                Value::String(format!("mine {round}")),
            );
            source.change_elsewhere(1, |item| change.clone_into(&mut item.note));
            ctx.reload();
            settle();

            // The values of the fields belong to the form, not to the effect that loaded them.
            for typed in ["a", "ab"] {
                enter(&edit.form, ItemField::Name, Value::String(typed.to_owned()));
                assert_that!(text_of(&edit.form, ItemField::Name))
                    .is_equal_to(Some(typed.to_owned()));
            }
            assert_that!(text_of(&edit.form, ItemField::Note)).is_equal_to(Some(change.to_owned()));
        }
    });
}

#[test]
fn a_reload_keeps_the_input_after_the_user_changed_an_id_field() {
    let source = MemorySource::with_items(1, MemoryPaging::Offset);
    with_config(source.config(), |ctx| {
        let edit = use_crud_edit_form(edit_input(item(1).id()));
        settle();
        enter(&edit.form, ItemField::Id, Value::I64(99));
        enter(
            &edit.form,
            ItemField::Name,
            Value::String("mine".to_owned()),
        );
        source.change_elsewhere(1, |item| "theirs".clone_into(&mut item.note));
        ctx.reload();
        settle();

        assert_that!(text_of(&edit.form, ItemField::Name)).is_equal_to(Some("mine".to_owned()));
        assert_that!(text_of(&edit.form, ItemField::Note)).is_equal_to(Some("theirs".to_owned()));
        let id = edit
            .form
            .field(&DynUpdateField::from(ItemField::Id))
            .map(|id| id.get_untracked());
        assert_that!(matches!(id, Some(Value::I64(99)))).is_true();
    });
}

#[test]
fn outdated_refusals_of_versions_that_a_reload_replaced_are_ignored() {
    let source = MemorySource::with_items(1, MemoryPaging::Offset);
    let notifications = Arc::new(Mutex::new(Vec::new()));
    with_crud_manager(|| {
        record_notifications(&notifications);
        provide_crud_instance(instance_input("items", source.config()));
        let edit = use_crud_edit_form(edit_input(item(1).id()));
        settle();
        enter(
            &edit.form,
            ItemField::Name,
            Value::String("mine".to_owned()),
        );
        source.change_elsewhere(1, |item| "theirs".clone_into(&mut item.note));
        let release = source.hold_next_update();
        edit.save.run(CrudSaveFollowUp::Stay);
        settle();
        assert_that!(edit.is_saving.get_untracked()).is_true();

        // The reload brings the version that the save in flight will be refused for.
        edit.reload.run(());
        settle();
        release.send(()).expect("the update waits");
        settle();
        assert_that!(edit.form.save_error.get_untracked()).is_none();

        edit.save.run(CrudSaveFollowUp::Stay);
        settle();
        assert_that!(edit.form.is_dirty.get_untracked()).is_false();
    });

    assert_that!(kinds_and_messages(&notifications)).is_equal_to(vec![(
        CrudNotificationKind::Success,
        english(CrudText::SaveUpdated),
    )]);
}

#[test]
fn failures_that_a_form_explains_are_not_also_notified() {
    let source = MemorySource::with_items(1, MemoryPaging::Offset);
    let notifications = Arc::new(Mutex::new(Vec::new()));
    with_crud_manager(|| {
        record_notifications(&notifications);
        provide_crud_instance(instance_input("items", source.config()));
        let edit = use_crud_edit_form(edit_input(item(1).id()));
        settle();
        let save_as = |name: &str| {
            enter(&edit.form, ItemField::Name, Value::String(name.to_owned()));
            edit.save.run(CrudSaveFollowUp::Stay);
            settle();
        };

        // As a mounted `CrudFormError` does.
        let summary = Owner::new();
        summary.with(|| crate::hooks::form::register_error_summary(edit.form.error_summaries()));
        save_as(TAKEN_NAME);
        save_as("free");
        drop(summary);
        save_as(TAKEN_NAME);
    });

    let kinds = kinds_and_messages(&notifications)
        .into_iter()
        .map(|(kind, _)| kind)
        .collect::<Vec<_>>();
    assert_that!(kinds).is_equal_to(vec![
        CrudNotificationKind::Success,
        CrudNotificationKind::Error,
    ]);
}

#[test]
fn switching_the_edited_entity_loads_its_values() {
    let source = MemorySource::with_items(2, MemoryPaging::Offset);
    with_config(source.config(), |_ctx| {
        let id = RwSignal::new(item(1).id());
        let edit = use_crud_edit_form(UseCrudEditFormInput {
            id: id.into(),
            ..edit_input(item(1).id())
        });
        for shown in [1, 2, 1] {
            id.set(item(shown).id());
            settle();
            assert_that!(text_of(&edit.form, ItemField::Name))
                .is_equal_to(Some(format!("item {shown}")));
        }
        enter(
            &edit.form,
            ItemField::Name,
            Value::String("typed".to_owned()),
        );
        assert_that!(text_of(&edit.form, ItemField::Name)).is_equal_to(Some("typed".to_owned()));
    });
}

#[test]
fn changes_that_cannot_apply_to_the_current_version_keep_the_form_outdated() {
    let source = MemorySource::with_items(1, MemoryPaging::Offset);
    source.change_elsewhere(1, |item| item.tags = serde_json::json!({ "a": "1" }));
    with_config(source.config(), |_ctx| {
        let edit = use_crud_edit_form(edit_input(item(1).id()));
        settle();
        // A merge patch removes a member set to `null`, but items require their tags.
        enter(
            &edit.form,
            ItemField::Tags,
            Value::Json(serde_json::Value::Null),
        );
        source.change_elsewhere(1, |item| "theirs".clone_into(&mut item.note));
        edit.save.run(CrudSaveFollowUp::Stay);
        settle();
        let alerts = edit.form.alerts.get_untracked();

        edit.reload.run(());
        settle();
        assert_that!(edit.form.rebase_failed.get_untracked()).is_true();
        assert_that!(matches!(
            edit.form.save_error.get_untracked(),
            Some(RequestError::Outdated(_))
        ))
        .is_true();
        assert_that!(edit.form.alerts.get_untracked()).is_equal_to(alerts + 1);
        // The draft and the version it is based on stay.
        assert_that!(json_of(&edit.form, ItemField::Tags))
            .is_equal_to(Some(serde_json::Value::Null));
        assert_that!(text_of(&edit.form, ItemField::Note)).is_equal_to(Some(String::new()));

        // Saving is refused again instead of replacing the other change.
        edit.save.run(CrudSaveFollowUp::Stay);
        settle();
        assert_that!(matches!(
            edit.form.save_error.get_untracked(),
            Some(RequestError::Outdated(_))
        ))
        .is_true();
    });

    let stored = source.item(1).expect("the item");
    assert_that!((stored.tags, stored.note))
        .is_equal_to((serde_json::json!({ "a": "1" }), "theirs".to_owned()));
    let versions = update_requests(&source)
        .iter()
        .map(|request| request.version().cloned())
        .collect::<Vec<_>>();
    assert_that!(versions).is_equal_to(vec![Some(CrudVersion::new("2")); 2]);
}

#[test]
fn every_failed_save_asks_for_attention_also_one_like_the_last() {
    let source = MemorySource::with_items(1, MemoryPaging::Offset);
    with_config(source.config(), |_ctx| {
        let edit = use_crud_edit_form(edit_input(item(1).id()));
        settle();
        enter(
            &edit.form,
            ItemField::Name,
            Value::String(TAKEN_NAME.to_owned()),
        );
        let mut alerts = vec![edit.form.alerts.get_untracked()];
        for _ in 0..2 {
            edit.save.run(CrudSaveFollowUp::Stay);
            settle();
            alerts.push(edit.form.alerts.get_untracked());
        }
        // Error summaries take the focus whenever the count grows.
        assert_that!(alerts).is_equal_to(vec![0, 1, 2]);
    });
}

#[test]
fn conflicts_follow_the_other_change_and_end_when_it_meets_the_users() {
    let source = MemorySource::with_items(1, MemoryPaging::Offset);
    with_config(source.config(), |ctx| {
        let edit = use_crud_edit_form(edit_input(item(1).id()));
        settle();
        enter(
            &edit.form,
            ItemField::Name,
            Value::String("mine".to_owned()),
        );
        let conflicts = move || {
            edit.form
                .conflicts
                .get_untracked()
                .into_iter()
                .map(|conflict| (conflict.pointer, conflict.theirs))
                .collect::<Vec<_>>()
        };
        let alerts = edit.form.alerts.get_untracked();

        for other in ["theirs", "third"] {
            source.change_elsewhere(1, |item| other.clone_into(&mut item.name));
            ctx.reload();
            settle();
            assert_that!(conflicts())
                .is_equal_to(vec![("/name".to_owned(), Some(serde_json::json!(other)))]);
        }
        // Only the new conflict asked for attention, not the one that changed its value.
        assert_that!(edit.form.alerts.get_untracked()).is_equal_to(alerts + 1);

        source.change_elsewhere(1, |item| "mine".clone_into(&mut item.name));
        ctx.reload();
        settle();
        assert_that!(conflicts()).is_empty();
        assert_that!(text_of(&edit.form, ItemField::Name)).is_equal_to(Some("mine".to_owned()));
    });
}

#[test]
fn fields_shown_twice_stay_registered_while_one_is_mounted() {
    let source = MemorySource::with_items(1, MemoryPaging::Offset);
    with_config(source.config(), |_ctx| {
        let edit = use_crud_edit_form(edit_input(item(1).id()));
        let target = |dom_id: &str| crate::hooks::form::FieldTarget {
            dom_id: leptonic::Id::from(dom_id.to_owned()),
            label: None,
            tabs: Vec::new(),
        };
        let mounted = move || {
            edit.form
                .field_targets()
                .get_untracked()
                .get("name")
                .map(|targets| {
                    targets
                        .iter()
                        .map(|target| target.dom_id.as_str().to_owned())
                        .collect::<Vec<_>>()
                })
        };
        let first = Owner::new();
        first.with(|| edit.form.register_field("name".to_owned(), target("first")));
        let second = Owner::new();
        second.with(|| {
            edit.form
                .register_field("name".to_owned(), target("second"));
        });
        assert_that!(mounted()).is_equal_to(Some(vec!["first".to_owned(), "second".to_owned()]));

        drop(first);
        assert_that!(mounted()).is_equal_to(Some(vec!["second".to_owned()]));
        drop(second);
        assert_that!(mounted()).is_none();
    });
}

#[test]
fn emptied_required_text_stays_while_the_value_changes() {
    with_crud_manager(|| {
        let (before, after) = (uuid::Uuid::from_u128(1), uuid::Uuid::from_u128(2));
        let value = ReactiveField::new(Value::Uuid(before));
        let reported = Arc::new(Mutex::new(Vec::new()));
        let sink = reported.clone();
        provide_context(CrudFieldBinding {
            name: "owner".to_owned(),
            value_kind: crudkit_core::ValueKind::Uuid,
            is_optional: false,
            value,
            mode: FieldMode::Editable,
            options: FieldOptions::default(),
            set: Callback::new(move |result: Result<Value, CrudFieldInputError>| {
                sink.lock().expect("lock").push(result.is_ok());
            }),
            error: Signal::stored(None),
            is_saving: Signal::stored(false),
            dom_id: leptonic::Id::from("owner"),
            has_renderer: false,
        });
        let input = use_crud_text_input(CrudTextCodec::uuid());
        settle();

        input.set_value.run(String::new());
        // E.g. a reload brings another value, while the field cannot read its input.
        value.set(Value::Uuid(after));
        settle();

        assert_that!(reported.lock().expect("lock").clone()).is_equal_to(vec![false]);
        assert_that!(input.value.get_untracked()).is_equal_to(String::new());
    });
}

/// Translates the texts of deletions and failed saves into German, and leaves every other text to
/// its English default.
fn german_notifications(text: &CrudText<'_>) -> Option<String> {
    Some(match *text {
        CrudText::DeleteCompletedTitle { count } => match count {
            1 => "Eintrag gelöscht".to_owned(),
            _ => "Einträge gelöscht".to_owned(),
        },
        CrudText::DeleteCompleted { count } => match count {
            1 => "1 Eintrag gelöscht.".to_owned(),
            _ => format!("{count} Einträge gelöscht."),
        },
        CrudText::SaveUpdateFailedTitle => "Änderungen nicht gespeichert".to_owned(),
        CrudText::ErrorInvalidResponse => "Ungültige Antwort".to_owned(),
        _ => return None,
    })
}

#[test]
fn notifications_show_the_texts_of_the_provided_translator() {
    let server = FakeServer::with_items(3);
    let notifications = Arc::new(Mutex::new(Vec::<CrudNotification>::new()));
    with_crud_manager(|| {
        record_notifications(&notifications);
        provide_crud_translator(german_notifications);
        provide_crud_instance(instance_input("items", config(&server)));

        let deletion = use_crud_delete();
        deletion.request_many(vec![item(1), item(3)]);
        deletion.confirm_many();
        settle();

        let edit = use_crud_edit_form(edit_input(item(2).id()));
        settle();
        edit.form.set_field(
            DynUpdateField::from(ItemField::Name),
            Ok(Value::String("reject".to_owned())),
        );
        edit.save.run(CrudSaveFollowUp::Stay);
        settle();
    });

    let titles_and_messages = notifications
        .lock()
        .expect("lock")
        .iter()
        .map(|notification| (notification.title.clone(), notification.message.clone()))
        .collect::<Vec<_>>();
    assert_that!(titles_and_messages).has_length(2);
    assert_that!(titles_and_messages[0].clone()).is_equal_to((
        "Einträge gelöscht".to_owned(),
        "2 Einträge gelöscht.".to_owned(),
    ));
    let (title, message) = titles_and_messages[1].clone();
    assert_that!(title).is_equal_to("Änderungen nicht gespeichert".to_owned());
    // The translator leaves the composition with the technical detail to its English default.
    assert_that!(message.as_str()).starts_with("Ungültige Antwort: ");
}

#[test]
fn rejected_input_is_explained_by_the_provided_translator() {
    with_crud_manager(|| {
        provide_crud_translator(|text: &CrudText<'_>| match *text {
            CrudText::InputInvalidUuid => Some("Gib eine UUID ein.".to_owned()),
            _ => None,
        });
        let reported = Arc::new(Mutex::new(Vec::new()));
        let sink = reported.clone();
        provide_context(CrudFieldBinding {
            name: "owner".to_owned(),
            value_kind: crudkit_core::ValueKind::Uuid,
            is_optional: false,
            value: ReactiveField::new(Value::Uuid(uuid::Uuid::from_u128(1))),
            mode: FieldMode::Editable,
            options: FieldOptions::default(),
            set: Callback::new(move |result: Result<Value, CrudFieldInputError>| {
                if let Err(error) = result {
                    sink.lock().expect("lock").push(error.to_string());
                }
            }),
            error: Signal::stored(None),
            is_saving: Signal::stored(false),
            dom_id: leptonic::Id::from("owner"),
            has_renderer: false,
        });
        let input = use_crud_text_input(CrudTextCodec::uuid());

        input.set_value.run("not a uuid".to_owned());

        assert_that!(reported.lock().expect("lock").clone())
            .is_equal_to(vec!["Gib eine UUID ein.".to_owned()]);
    });
}

#[test]
fn texts_follow_the_language_that_the_translator_reads() {
    with_crud_manager(|| {
        let german = RwSignal::new(false);
        provide_crud_translator(move |text: &CrudText<'_>| match *text {
            CrudText::FormSave if german.get() => Some("Speichern".to_owned()),
            _ => None,
        });
        let texts = use_crud_texts();
        let label = Memo::new(move |_| texts.get(CrudText::FormSave));
        assert_that!(label.get_untracked()).is_equal_to("Save".to_owned());

        german.set(true);

        assert_that!(label.get_untracked()).is_equal_to("Speichern".to_owned());
    });
}
