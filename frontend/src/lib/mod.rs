mod state;

use anyhow::Error;
use state::{Filter, State};
use std::string::ToString;
use strum::IntoEnumIterator;
use todomvc_shared::{Entries, Entry, TaskRequest};
use uuid::Uuid;
use yew::{
    classes,
    events::KeyboardEvent,
    format::{MsgPack, Nothing},
    html,
    services::{
        fetch::{FetchTask, Request, Response},
        ConsoleService, FetchService,
    },
    web_sys::HtmlInputElement as InputElement,
    Callback, Classes, Component, ComponentLink, Html, InputData, NodeRef, ShouldRender,
};

type FetchResponse<T> = Response<MsgPack<Result<T, Error>>>;
pub enum Msg {
    Add,
    Edit(usize),
    Update(String),
    UpdateEdit(String),
    Remove(usize),
    SetFilter(Filter),
    ToggleAll,
    ToggleEdit(usize),
    Toggle(usize),
    ClearCompleted,
    Focus,
    FetchError(FetchErrorType),
    FetchOk(FetchOkType),
}

pub enum FetchErrorType {
    Meta(u16, Option<String>),
    Data(String),
}

pub enum FetchOkType {
    Entries(Entries),
    Entry(Entry),
    NoData,
}

pub struct Model {
    link: ComponentLink<Self>,
    state: State,
    focus_ref: NodeRef,
    ft: Option<FetchTask>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let entries = Vec::new();
        let _task = match fetch_all_tasks(&link) {
            Ok(t) => Some(t),
            Err(e) => {
                ConsoleService::error(
                    format!("Initial fetch task failed, reason: {}", e.to_string()).as_str(),
                );
                None
            }
        };
        let state = State {
            entries,
            filter: Filter::All,
            value: "".into(),
            edit_value: "".into(),
        };
        let focus_ref = NodeRef::default();
        Self {
            link,
            state,
            focus_ref,
            ft: _task,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Add => {
                let content = self.state.value.trim();
                let mut sr = false;
                if !content.is_empty() {
                    let ft = self.create_task(content);
                    sr = self.handle_ft(ft, "create task");
                }
                self.state.value = "".to_string();
                sr
            }
            Msg::Edit(idx) => {
                let edit_value = self.state.edit_value.trim().to_string();
                let (e, is_remove) = self.state.complete_edit(idx, edit_value);
                let (ft, desc) = if is_remove {
                    (self.remove_task(*e.id()), "remove task")
                } else {
                    (self.update_task(&e), "finish edit task")
                };
                self.state.edit_value = "".to_string();
                self.handle_ft(ft, desc)
            }
            Msg::Update(val) => {
                self.state.value = val;
                true
            }
            Msg::UpdateEdit(val) => {
                self.state.edit_value = val;
                true
            }
            Msg::Remove(idx) => {
                let e = self.state.remove(idx);
                let id = *e.id();
                self.handle_ft(self.remove_task(id), "remove task")
            }
            Msg::SetFilter(filter) => {
                self.state.filter = filter;
                true
            }
            Msg::ToggleEdit(idx) => {
                self.state.edit_value = self.state.entries[idx].content().clone();
                self.state.clear_all_edit();
                let e = self.state.toggle_edit(idx);
                self.handle_ft(self.update_task(&e), "toggle edit")
            }
            Msg::ToggleAll => {
                let status = !self.state.is_all_completed();
                self.state.toggle_all(status);
                self.handle_ft(self.update_all_tasks(), "toggle all tasks as completed")
            }
            Msg::Toggle(idx) => {
                let e = self.state.toggle(idx);
                self.handle_ft(self.update_task(&e), "toggle completed")
            }
            Msg::ClearCompleted => {
                self.state.clear_completed();
                self.handle_ft(self.update_all_tasks(), "clear all completed tasks")
            }
            Msg::Focus => {
                if let Some(input) = self.focus_ref.cast::<InputElement>() {
                    input.focus().unwrap();
                }
                true
            }
            Msg::FetchOk(t) => match t {
                FetchOkType::Entries(es) => {
                    ConsoleService::info("Fetch entries success.");
                    self.state.entries = es;
                    true
                }
                FetchOkType::Entry(e) => {
                    ConsoleService::info("Create entry success.");
                    self.state.entries.push(e);
                    true
                }
                FetchOkType::NoData => {
                    ConsoleService::info("Fetch success");
                    false
                }
            },
            Msg::FetchError(kind) => {
                match kind {
                    FetchErrorType::Data(reason) => ConsoleService::error(
                        format!("Error fetching data, reason: {}", reason).as_str(),
                    ),
                    FetchErrorType::Meta(code, reason) => ConsoleService::error(
                        format!(
                            "Got status code: {}, reason: {}",
                            code,
                            match reason {
                                Some(r) => r,
                                None => "No reason".to_string(),
                            }
                        )
                        .as_str(),
                    ),
                };
                false
            }
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let hidden_class = if self.state.entries.is_empty() {
            "hidden"
        } else {
            ""
        };
        html! {
            <div class="todomvc-wrapper">
                <section class="todoapp">
                    <header class="header">
                        <h1>{ "todos" }</h1>
                        { self.view_input() }
                    </header>
                    <section class=classes!("main", hidden_class)>
                        <input
                            type="checkbox"
                            class="toggle-all"
                            id="toggle-all"
                            checked=self.state.is_all_completed()
                            onclick=self.link.callback(|_| Msg::ToggleAll)
                        />
                        <label for="toggle-all" />
                        <ul class="todo-list">
                            { for self.state.entries.iter().filter(|e| self.state.filter.fits(e)).enumerate().map(|e| self.view_entry(e)) }
                        </ul>
                    </section>
                    <footer class=classes!("footer", hidden_class)>
                        <span class="todo-count">
                            <strong>{ self.state.total() }</strong>
                            { " item(s) left" }
                        </span>
                        <ul class="filters">
                            { for Filter::iter().map(|flt| self.view_filter(flt)) }
                        </ul>
                        <button class="clear-completed" onclick=self.link.callback(|_| Msg::ClearCompleted)>
                            { format!("Clear completed ({})", self.state.total_completed()) }
                        </button>
                    </footer>
                </section>
                <footer class="info">
                    <p>{ "Double-click to edit a todo" }</p>
                    <p>{ "Written by " }<a href="https://github.com/DenisKolodin/" target="_blank">{ "Denis Kolodin" }</a></p>
                    <p>{ "Part of " }<a href="http://todomvc.com/" target="_blank">{ "TodoMVC" }</a></p>
                </footer>
            </div>
        }
    }
}

impl Model {
    fn view_filter(&self, filter: Filter) -> Html {
        let cls = if self.state.filter == filter {
            "selected"
        } else {
            "not-selected"
        };
        html! {
            <li>
                <a class=cls
                   href=filter.as_href()
                   onclick=self.link.callback(move |_| Msg::SetFilter(filter))
                >
                    { filter }
                </a>
            </li>
        }
    }

    fn view_input(&self) -> Html {
        html! {
            // You can use standard Rust comments. One line:
            // <li></li>
            <input
                class="new-todo"
                placeholder="What needs to be done?"
                value=self.state.value.clone()
                oninput=self.link.callback(|e: InputData| Msg::Update(e.value))
                onkeypress=self.link.batch_callback(|e: KeyboardEvent| {
                    if e.key() == "Enter" { Some(Msg::Add) } else { None }
                })
            />
            /* Or multiline:
            <ul>
                <li></li>
            </ul>
            */
        }
    }

    fn view_entry(&self, (idx, entry): (usize, &Entry)) -> Html {
        let mut class = Classes::from("todo");
        if *entry.editing() {
            class.push(" editing");
        }
        if *entry.completed() {
            class.push(" completed");
        }
        html! {
            <li class=class>
                <div class="view">
                    <input
                        type="checkbox"
                        class="toggle"
                        checked=*entry.completed()
                        onclick=self.link.callback(move |_| Msg::Toggle(idx))
                    />
                    <label ondblclick=self.link.callback(move |_| Msg::ToggleEdit(idx))>{ entry.content() }</label>
                    <button class="destroy" onclick=self.link.callback(move |_| Msg::Remove(idx)) />
                </div>
                { self.view_entry_edit_input((idx, entry)) }
            </li>
        }
    }

    fn view_entry_edit_input(&self, (idx, entry): (usize, &Entry)) -> Html {
        if *entry.editing() {
            html! {
                <input
                    class="edit"
                    type="text"
                    ref=self.focus_ref.clone()
                    value=self.state.edit_value.clone()
                    onmouseover=self.link.callback(|_| Msg::Focus)
                    oninput=self.link.callback(|e: InputData| Msg::UpdateEdit(e.value))
                    onblur=self.link.callback(move |_| Msg::Edit(idx))
                    onkeypress=self.link.batch_callback(move |e: KeyboardEvent| {
                        if e.key() == "Enter" { Some(Msg::Edit(idx)) } else { None }
                    })
                />
            }
        } else {
            html! { <input type="hidden" /> }
        }
    }
    fn update_all_tasks(&self) -> Result<FetchTask, Error> {
        let data = MsgPack(&self.state.entries);
        let request = build_request("POST", "/tasks", data);
        FetchService::fetch_binary(request, self.fetch_callback())
    }
    fn update_task(&self, e: &Entry) -> Result<FetchTask, Error> {
        let id = e.id().to_hyphenated();
        let id_str = id
            .encode_lower(&mut uuid::Uuid::encode_buffer())
            .to_string();
        let data = MsgPack(e);
        let request = build_request("PUT", format!("/task?id={}", id_str), data);
        FetchService::fetch_binary(request, self.fetch_callback())
    }
    fn create_task(&self, content: &str) -> Result<FetchTask, Error> {
        let tr = TaskRequest { content };
        let data = MsgPack(&tr);
        let request = build_request("POST", "/task", data);
        FetchService::fetch_binary(request, self.create_callback())
    }
    fn remove_task(&self, eid: Uuid) -> Result<FetchTask, Error> {
        let id = eid.to_hyphenated();
        let id_str = id
            .encode_lower(&mut uuid::Uuid::encode_buffer())
            .to_string();
        let request = build_request("DELETE", format!("/task?id={}", id_str), Nothing);
        FetchService::fetch_binary(request, self.fetch_callback())
    }
    fn fetch_callback(&self) -> Callback<FetchResponse<()>> {
        self.link.callback(move |resp: FetchResponse<()>| {
            let (meta, _) = resp.into_parts();
            if meta.status.is_success() {
                Msg::FetchOk(FetchOkType::NoData)
            } else {
                let status = meta.status;
                Msg::FetchError(FetchErrorType::Meta(
                    status.as_u16(),
                    status.canonical_reason().map(|s| s.to_string()),
                ))
            }
        })
    }
    fn create_callback(&self) -> Callback<FetchResponse<Entry>> {
        self.link.callback(move |resp: FetchResponse<Entry>| {
            let (meta, MsgPack(r)) = resp.into_parts();
            if meta.status.is_success() {
                match r {
                    Ok(e) => Msg::FetchOk(FetchOkType::Entry(e)),
                    Err(e) => Msg::FetchError(FetchErrorType::Data(e.to_string())),
                }
            } else {
                let status = meta.status;
                Msg::FetchError(FetchErrorType::Meta(
                    status.as_u16(),
                    status.canonical_reason().map(|s| s.to_string()),
                ))
            }
        })
    }
    fn handle_ft(&mut self, ft: Result<FetchTask, Error>, task_desc: &str) -> ShouldRender {
        match ft {
            Ok(t) => {
                self.ft = Some(t);
                true
            }
            Err(e) => {
                ConsoleService::error(
                    format!("Error, occur: {}, reason: {}", task_desc, e.to_string()).as_str(),
                );
                self.ft = None;
                false
            }
        }
    }
}

fn fetch_all_tasks(link: &ComponentLink<Model>) -> Result<FetchTask, Error> {
    let callback = link.callback(move |resp: Response<MsgPack<Result<Entries, Error>>>| {
        let (meta, MsgPack(data)) = resp.into_parts();
        // ConsoleService::log(format!("META: {:?}, {:?}", meta, data).as_str());
        if meta.status.is_success() {
            match data {
                Ok(v) => Msg::FetchOk(FetchOkType::Entries(v)),
                Err(e) => Msg::FetchError(FetchErrorType::Data(e.to_string())),
            }
        } else {
            let status = meta.status;
            Msg::FetchError(FetchErrorType::Meta(
                status.as_u16(),
                status.canonical_reason().map(|s| s.to_string()),
            ))
        }
    });
    // let request = Request::get("/tasks").body(Nothing).unwrap();
    let request = build_request("GET", "/tasks", Nothing);
    FetchService::fetch_binary(request, callback)
}

fn build_request<T, U: ToString>(method: &str, uri: U, data: T) -> Request<T> {
    Request::builder()
        .method(method)
        .uri(uri.to_string())
        .header("Content-Type", "application/msgpack")
        .body(data)
        .unwrap()
}
