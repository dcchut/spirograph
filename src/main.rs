use gloo_timers::callback::Interval;
use material_yew::MatSlider;
use serde::Deserialize;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

struct SpirographIter {
    s: Spirograph,
    now: f64,
    width: f64,
}

impl Iterator for SpirographIter {
    type Item = (f64, f64);

    fn next(&mut self) -> Option<Self::Item> {
        let (x, y) = self.s.at(self.now);
        self.now += self.width;
        Some((x, y))
    }
}

// Follows the notation of https://en.wikipedia.org/wiki/Spirograph#Mathematical_basis
#[derive(Copy, Clone, Debug)]
struct Spirograph {
    // Distance of defining point from centre of inner circle.
    l: f64,

    // Ratio of size of inner circle with respect to the outer one.
    k: f64,

    // Radius of the outer circle
    r: f64,
}

impl Spirograph {
    pub fn new(l: f64, k: f64, r: f64) -> Self {
        Self { l, k, r }
    }

    pub fn iter(self, width: f64) -> SpirographIter {
        SpirographIter {
            s: self,
            now: 0.0,
            width,
        }
    }

    #[inline(always)]
    pub fn at(self, t: f64) -> (f64, f64) {
        let x = self.r
            * ((1. - self.k) * t.cos() + self.l * self.k * (t * (1. - self.k) / self.k).cos());
        let y = self.r
            * ((1. - self.k) * t.sin() - self.l * self.k * (t * (1. - self.k) / self.k).sin());
        (x, y)
    }
}

// # def spirograph(t, k=math.pi/10, l=0.7, R=250.0):
// x = R * ((1 - k) * math.cos(t) + l*k*math.cos(t * ((1-k)/k)))
// y = R * ((1 - k) * math.sin(t) - l*k*math.sin(t * ((1-k)/k)))

pub struct Canvas {
    pub canvas: HtmlCanvasElement,
    pub context: CanvasRenderingContext2d,
}

impl Canvas {
    pub fn new() -> Self {
        let canvas: HtmlCanvasElement = gloo_utils::document()
            .create_element("canvas")
            .unwrap()
            .unchecked_into();
        canvas.set_width(500);
        canvas.set_height(500);

        let context: CanvasRenderingContext2d =
            canvas.get_context("2d").unwrap().unwrap().unchecked_into();
        context.begin_path();

        Self { canvas, context }
    }

    pub fn line_to(&self, x: f64, y: f64) {
        self.context.line_to(x, y);
    }

    pub fn move_to(&self, x: f64, y: f64) {
        self.context.move_to(x, y);
    }

    pub fn stroke(&self) {
        self.context.stroke();
    }
}

enum Msg {
    Tick,
    LSlider(f64),
    KSlider(f64),
}

struct Model {
    canvas: Canvas,
    spirograph: SpirographIter,
    _interval: Interval,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let canvas = Canvas::new();
        let interval = {
            let link = ctx.link().clone();
            Interval::new(12, move || link.send_message(Msg::Tick))
        };

        Self {
            canvas,
            spirograph: Spirograph::new(0.22, 0.46, 150.).iter(0.15),
            _interval: interval,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => {
                let (x, y) = self.spirograph.next().unwrap();
                self.canvas.context.line_to(250. + x, 250. + y);
                self.canvas.context.stroke();
            }
            Msg::LSlider(l) => {
                self.spirograph.s.l = l;
                self.spirograph.now = 0.;
                self.canvas = Canvas::new();
            }
            Msg::KSlider(k) => {
                self.spirograph.s.k = k;
                self.spirograph.now = 0.;
                self.canvas = Canvas::new();
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let cb = |e: web_sys::CustomEvent| -> f64 {
            #[derive(Debug, Deserialize)]
            struct SlideEventDetails {
                #[serde(rename = "_value")]
                value: f64,
            }
            let obj: SlideEventDetails = e.detail().into_serde().unwrap();

            (obj.value / 100.).clamp(0.01, 0.99)
        };

        let onslide_l = ctx.link().callback(move |e| Msg::LSlider(cb(e)));
        let onslide_k = ctx.link().callback(move |e| Msg::KSlider(cb(e)));

        html! {
            <div>
                { Html::VRef(self.canvas.canvas.to_owned().into()) }
                <br />
                <div><b>{ "k" }</b><MatSlider value={46} oninput={onslide_k} /></div>
                <div><b>{ "l" }</b><MatSlider value={22} oninput={onslide_l} /></div>
            </div>
        }
    }

    // fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
    //     if !first_render {
    //         return;
    //     }
    //
    //     // Add button onclick handler
    //     if let Some(element) = self.button_ref.cast::<HtmlElement>() {
    //         let onclick = ctx.link().callback(|_| Msg::Peon);
    //         let listener = EventListener::new(
    //             &element,
    //             "click",
    //             move |e| onclick.emit(e.clone())
    //         );
    //         self.button_click_listener = Some(listener);
    //     }
    // }
}

fn main() {
    yew::start_app::<Model>();
}
