//! Tests of the state hooks against an in-memory CrudKit server.

use crate::config::CrudInstanceConfig;
use crate::hooks::form::{
    CrudSaveFollowUp, UseCrudCreateFormInput, UseCrudEditFormInput, use_crud_create_form,
    use_crud_edit_form,
};
use crate::hooks::instance::use_crud_navigation;
use crate::hooks::list::use_crud_list_state;
use crate::hooks::notify::{
    CrudNotification, CrudNotificationKind, CrudNotifier, provide_crud_notifier,
};
use crate::instance::{provide_crud_instance, provide_view_context};
use crate::prelude::*;
use crate::test_support::items::*;
use crate::test_support::{CrudTestResponse, CrudTestServer, settle, with_crud_manager};
use assertr::prelude::*;
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
            .run(leptonic::hooks::Key::from("contact"));
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
fn deletions_stay_within_the_instance_scope() {
    let server = FakeServer::with_items(3);
    let mut scope = crudkit_core::condition::Condition::all();
    scope.push_elements(vec![crudkit_core::condition::ConditionElement::Clause(
        crudkit_core::condition::ConditionClause {
            column_name: "name".to_owned(),
            operator: crudkit_core::condition::Operator::Equal,
            value: crudkit_core::condition::ConditionClauseValue::String("scoped".to_owned()),
        },
    )]);
    with_crud_manager(|| {
        let config = CrudInstanceConfig {
            base_condition: Some(scope),
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
            id: 1,
            name: "renamed".to_owned(),
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
