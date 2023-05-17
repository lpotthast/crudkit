use yew::{html::ChildrenRenderer, prelude::*, virtual_dom::VChild};

use super::prelude::*;
use yew_bootstrap_icons::v1_10_3::Bi;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Small,
    Normal,
    Big,
}

// TODO: This con be computed statically!
impl From<Size> for Classes {
    fn from(size: Size) -> Self {
        match size {
            Size::Small => classes!("small"),
            Size::Normal => classes!("normal"), // TODO: is this even necessary?
            Size::Big => classes!("big"),
        }
    }
}

pub enum Msg {
    Clicked(MouseEvent),
    DropdownTriggerClicked,
}

#[derive(Clone, derive_more::From, PartialEq)]
pub enum Item {
    Name(VChild<CrudBtnName>),
    AlternativeAction(VChild<CrudBtn>),
}

// Now, we implement `Into<Html>` so that yew knows how to render `Item`.
#[allow(clippy::from_over_into)]
impl Into<Html> for Item {
    fn into(self) -> Html {
        match self {
            Item::Name(child) => child.into(),
            Item::AlternativeAction(child) => child.into(),
        }
    }
}

#[derive(Debug, PartialEq, Properties)]
pub struct CrudBtnProps {
    pub name: String,
    #[prop_or(Variant::Primary)]
    pub variant: Variant,
    #[prop_or(Size::Normal)]
    pub size: Size,
    #[prop_or(false)]
    pub active: bool,
    #[prop_or(false)]
    pub disabled: bool,
    #[prop_or_default]
    pub icon: Option<Bi>,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
    //pub custom_name: Option<VChild<CrudBtnName>>,
    #[prop_or_default]
    pub children: ChildrenRenderer<Item>,
}

pub struct CrudBtn {
    has_custom_name: bool,
    has_variations: bool,
    dropdown_open: bool,
}

impl CrudBtn {
    pub fn get_custom_name<'a>(ctx: &'a Context<Self>) -> impl Iterator<Item = Item> + 'a {
        ctx.props().children.iter().filter(|item| match item {
            Item::Name(_) => true,
            Item::AlternativeAction(_) => false,
        })
    }

    pub fn get_variations<'a>(ctx: &'a Context<Self>) -> impl Iterator<Item = Item> + 'a {
        ctx.props().children.iter().filter(|item| match item {
            Item::Name(_) => false,
            Item::AlternativeAction(_) => true,
        })
    }
}

pub fn inspect_children(items: ChildrenRenderer<Item>) -> (bool, bool) {
    let has_custom_name = items.iter().any(|item| match item {
        Item::Name(_) => true,
        Item::AlternativeAction(_) => false,
    });

    let has_variations = items.iter().any(|item| match item {
        Item::Name(_) => false,
        Item::AlternativeAction(_) => true,
    });

    (has_custom_name, has_variations)
}

impl Component for CrudBtn {
    type Message = Msg;
    type Properties = CrudBtnProps;

    fn create(ctx: &Context<Self>) -> Self {
        let (has_custom_name, has_variations) = inspect_children(ctx.props().children.clone());
        Self {
            has_custom_name,
            has_variations,
            dropdown_open: false,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        let (has_custom_name, has_variations) = inspect_children(ctx.props().children.clone());
        self.has_custom_name = has_custom_name;
        self.has_variations = has_variations;
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Clicked(mouse_event) => {
                if !ctx.props().disabled {
                    ctx.props().onclick.emit(mouse_event);
                }
                false
            }
            Msg::DropdownTriggerClicked => {
                self.dropdown_open = !self.dropdown_open;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div
                class={classes!(
                    "crud-btn",
                    ctx.props().variant.class_name(),
                    ctx.props().size,
                    ctx.props().active.then(|| "active"),
                    ctx.props().disabled.then(|| "disabled"),
                    self.has_variations.then(|| "has-variations")
                )}
            >
                <div class={"name"}
                     onclick={ctx.link().callback(|mouse_event| Msg::Clicked(mouse_event))}>
                    if let Some(bi) = ctx.props().icon {
                        <CrudIcon variant={bi}/>
                    }
                    if self.has_custom_name {
                        { for CrudBtn::get_custom_name(ctx) }
                    } else {
                        { &ctx.props().name }
                    }
                </div>

                if self.has_variations {
                    <div class="dropdown-trigger"
                         onclick={ctx.link().callback(|_| Msg::DropdownTriggerClicked)}>
                         if self.dropdown_open {
                            <CrudIcon variant={Bi::CaretUp} />
                         } else {
                            <CrudIcon variant={Bi::CaretDown} />
                         }
                    </div>

                    <div class={classes!(
                        "dropdown",
                        self.dropdown_open.then(|| "active")
                    )}>
                        { for CrudBtn::get_variations(ctx) }
                    </div>
                }
            </div>
        }
    }
}
