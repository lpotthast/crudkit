use yew::{prelude::*, html::Scope};

use crate::{types::{
    files::{FileResource, ListFilesResponse},
    RequestError,
}, services::requests::request_get};

pub enum Msg {
    ListFiles,
    ListFilesResponse(Result<ListFilesResponse, RequestError>),
    Selected(FileResource),
    Reload,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub api_base_url: String,
    pub show_file_names: bool,
    pub on_link: Option<Callback<Scope<CrudImageGallery>>>,
    pub on_select: Option<Callback<FileResource>>,
}

pub struct CrudImageGallery {
    resources: Vec<FileResource>,
}

impl Component for CrudImageGallery {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        if let Some(on_link) = &ctx.props().on_link {
            on_link.emit(ctx.link().clone());
        };
        ctx.link().send_message(Msg::ListFiles);
        Self {
            resources: Vec::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ListFiles => {
                let base = ctx.props().api_base_url.clone();
                ctx.link().send_future(async move {
                    Msg::ListFilesResponse(
                        request_get::<ListFilesResponse>(format!("{}/list", base))
                            .await,
                    )
                });
                false
            }
            Msg::ListFilesResponse(result) => {
                match result {
                    Ok(files) => {
                        if let Some(err) = files.error {
                            log::error!("Could not list files: {:?}", err);
                        } else {
                            self.resources = files.files;
                        }
                    }
                    Err(err) => log::error!("Could not list files: {}", err),
                }
                true
            }
            Msg::Selected(resource) => {
                if let Some(on_select) = &ctx.props().on_select {
                    on_select.emit(resource);
                }
                false
            }
            Msg::Reload => {
                ctx.link().send_message(Msg::ListFiles);
                false
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class={"gallery"}>
                {
                    self.resources.iter()
                        .map(|resource| {
                            let cloned = resource.clone();
                            html! {
                                <div class={"img-wrapper"}>
                                    <img
                                        src={format!("http://127.0.0.1:8080/api/public/{}", urlencoding::encode(resource.name.as_str()))}
                                        alt={resource.name.clone()}
                                        onclick={ctx.link().callback(move |_| Msg::Selected(cloned.clone()))}/>
                                    if ctx.props().show_file_names {
                                        <span>{resource.name.clone()}</span>
                                    }
                                </div>
                            }
                        })
                        .collect::<Html>()
                }
            </div>
        }
    }
}
