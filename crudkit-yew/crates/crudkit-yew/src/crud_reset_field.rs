use yew::prelude::*;

pub enum Msg {}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub children: Children,
    pub name: String,
    pub for_model: Model,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Model {
    Create,
    Update,
}

/// This component is used as a wrapper around select fields.
/// The CrudSelectField component is generic over the type it is used with.
/// But yew / the crud system can only distinguish between non-generic components...
pub struct CrudResetField {}

impl Component for CrudResetField {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            { for ctx.props().children.iter() }
        }
    }
}
