use leptos::*;

#[component]
pub fn CrudEditView(cx: Scope) -> impl IntoView {
    let (user_wants_to_leave, set_user_wants_to_leave) = create_signal(cx, false);
    
    view! {cx,
        "foo"
     
        <div>
            "fields"
        </div>

        
    }
}
