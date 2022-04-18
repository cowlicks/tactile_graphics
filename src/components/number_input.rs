use std::cell::Cell;
use web_sys::HtmlInputElement;
use yew::{events::InputEvent, html, Callback, Component, Context, Html, Properties, TargetCast};

thread_local! {
    static ELEMENT_ID: Cell<usize> = Cell::default();
}
fn next_element_id() -> usize {
    ELEMENT_ID.with(|cell| cell.replace(cell.get() + 1))
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub label: &'static str,
    pub value: f64,
    pub onchange: Callback<f64>,
    #[prop_or_default]
    pub precision: Option<usize>,
    #[prop_or_default]
    pub percentage: bool,
    #[prop_or_default]
    pub min: f64,
    pub max: f64,
}

pub struct NumberInput {
    id: usize,
}
impl Component for NumberInput {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            id: next_element_id(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        unimplemented!()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let Props {
            label,
            value,
            ref onchange,
            precision,
            percentage,
            min,
            max,
        } = *ctx.props();

        let precision = precision.unwrap_or(if percentage { 1 } else { 0 });

        let display_value = if percentage {
            format!("{:.p$}%", 100.0 * value, p = precision)
        } else {
            format!("{:.p$}", value, p = precision)
        };

        let id = format!("number-input-{}", self.id);

        let oninput = onchange.reform(|e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            input.value_as_number()
        });

        html! {
            <div class="number-input">
                <label for={id.clone()} class="number__label">{ label }</label>
                <input type="number"
                    {id}
                    class="number__input"
                    min={min.to_string()}
                    max={max.to_string()}
                    {oninput}
                />
                <span class="number__value">{ display_value }</span>
            </div>
        }
    }
}
