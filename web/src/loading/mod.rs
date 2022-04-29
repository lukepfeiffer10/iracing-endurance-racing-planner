use yew::prelude::*;

#[function_component(Loading)]
pub fn loading() -> Html {
    html! {
        <div class="loading">
            <img class="logo" src="android-chrome-512x512.png" height="64" width="64" />
            <div class="text" style="--i:1">{ "A" }</div>
            <div class="text" style="--i:2">{ "R" }</div>
            <div class="text" style="--i:3">{ "T" }</div>
            <div class="text" style="--i:4">{ "I" }</div>
            <div class="text" style="--i:5">{ "F" }</div>
            <div class="text" style="--i:6">{ "A" }</div>
            <div class="text" style="--i:7">{ "C" }</div>
            <div class="text" style="--i:8">{ "T" }</div>
            <div class="text" style="--i:9"></div>
            <div class="text" style="--i:10">{ "R" }</div>
            <div class="text" style="--i:11">{ "A" }</div>
            <div class="text" style="--i:12">{ "C" }</div>
            <div class="text" style="--i:13">{ "I" }</div>
            <div class="text" style="--i:14">{ "N" }</div>
            <div class="text" style="--i:15">{ "G" }</div>
        </div>
    }
}