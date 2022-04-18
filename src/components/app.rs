use std::rc::Rc;
use web_sys::{
    Event,
    HtmlInputElement
};
use yew::{html, html::TargetCast, Component, Context, Html};

use gloo_file::callbacks::FileReader;
use gloo_file::File;
use log::info;

use crate::components::image::img_html_from_bytes;
use crate::components::threshold::ThresholdImage;
use super::store::GlobalState;

use yewdux::prelude::*;

pub enum Msg {
    FileLoading,
    LoadedBytes(String, Vec<u8>),
    Files(Vec<File>),
    State(Rc<GlobalState>),
}


pub struct App {
    reader: Option<FileReader>,
    file_name: Option<String>,
    file_bytes: Option<Rc<Vec<u8>>>,
    file_loading: bool,
    _dispatch: Dispatch<BasicStore<GlobalState>>,
    state: Option<Rc<GlobalState>>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let _dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::State));
        Self {
            reader: None,
            file_name: None,
            file_bytes: None,
            file_loading: false,
            _dispatch,
            state: Default::default()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FileLoading => {
                self.file_loading = true;
                true
            }
            Msg::State(state) => {
                self.state = Some(state);
                true
            }
            Msg::LoadedBytes(file_name, data) => {
                info!("Loaded bytes");
                self.file_name = Some(file_name);
                self.file_bytes = Some(Rc::from(data));
                self.file_loading = false;
                self.reader = None;
                true
            }
            Msg::Files(files) => {
                for file in files.into_iter() {
                    let file_name = file.name();
                    let task = {
                        let file_name = file_name.clone();
                        let link = ctx.link().clone();

                        info!("Got filename = {}", file_name);
                        gloo_file::callbacks::read_as_bytes(&file, move |res| {
                            link.send_message(Msg::LoadedBytes(
                                file_name,
                                res.expect("failed to read file"),
                            ))
                        })
                    };
                    self.reader = Some(task);
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link().clone();
        html! {
            <div>
                <div>
                    <input type="file"
                        accept="image/png, image/jpeg"
                        multiple=false
                        onchange={ctx.link().callback(move |e: Event| {
                            link.send_message(Msg::FileLoading);
                            // send message here to indicate start file loading
                            let mut result = Vec::new();
                            let input: HtmlInputElement = e.target_unchecked_into();

                            if let Some(files) = input.files() {
                                let files = js_sys::try_iter(&files)
                                    .unwrap()
                                    .unwrap()
                                    .map(|v| web_sys::File::from(v.unwrap()))
                                    .map(File::from);
                                result.extend(files);
                            }
                            Msg::Files(result)
                        })}
                    />
                <div>
                    <h1>
                    if self.file_loading {
                        // TODO ADD LOADING STUFF
                        { "FILE LOADING ..." }
                    }
                    </h1>
                </div>
                </div>
                { self.view_file(&ctx) }
            </div>
        }
    }
}

impl App {
    fn view_file(&self, _ctx: &Context<Self>) -> Html {
        if let Some(data) = &self.file_bytes {
            html! {
                <div class="image-container">
                    <div class="images">
                        <div class="cell">
                            { img_html_from_bytes(data) }
                        </div>
                        <div class="cell">
                         <ThresholdImage
                                bytes={data}
                                />
                        </div>
                    </div>
                </div>
            }
        } else {
            html! {
                <p>{ "Choose a file to convert to stl" }</p>
            }
        }
    }
}
