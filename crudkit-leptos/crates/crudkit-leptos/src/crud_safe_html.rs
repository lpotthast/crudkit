use yew::{function_component, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub html: String,
}

#[function_component(CrudSafeHtml)]
pub fn safe_html(props: &Props) -> Html {
    let div = gloo::utils::document().create_element("span").unwrap();
    div.set_inner_html(&props.html.clone());
    Html::VRef(div.into())
}
