use crate::components::stl::StlViewer;

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
    dispatch: Dispatch<BasicStore<GlobalState>>,
    state: Option<Rc<GlobalState>>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let dispatch = Dispatch::bridge_state(ctx.link().callback(Msg::State));
        Self {
            reader: None,
            file_name: None,
            file_bytes: None,
            file_loading: false,
            dispatch,
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
                self.file_name = Some(file_name.clone());
                self.dispatch.reduce(move |state| state.file_name = Some(file_name.clone()));
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
                        <div class="lds-ring"><div></div><div></div><div></div><div></div></div>
                        { "FILE LOADING ..." }
                    }
                    </h1>
                </div>
                </div>
                if let Some(data) = &self.file_bytes {
                    <div class="image-container">
                        <div class="images">
                            <div class="cell">
                                { img_html_from_bytes(data) }
                            </div>
                            <div class="cell">
                             <ThresholdImage
                                    bytes={ Rc::clone(data) }
                                    />
                            if self.state.clone().map_or(false, |x| x.display_stl) {
                                <StlViewer
                                    bytes={ Rc::clone(data) }
                                />
                            }
                            </div>
                        </div>
                    </div>
                } else {
                    <p>{ "Choose a file to convert to stl" }</p>
                }
            </div>
        }
    }
}
